// Copyright 2018-2019 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod backup;
mod mem_pool;
mod mem_pool_types;
#[cfg_attr(feature = "cargo-clippy", allow(clippy::module_inception))]
mod miner;
mod sealing_queue;
mod stratum;
mod work_notify;

use self::mem_pool_types::AccountDetails;
pub use self::mem_pool_types::MemPoolMinFees;
pub use self::miner::{AuthoringParams, Miner, MinerOptions};
pub use self::stratum::{Config as StratumConfig, Error as StratumError, Stratum};
use crate::account_provider::{AccountProvider, Error as AccountProviderError};
use crate::block::ClosedBlock;
use crate::client::{
    AccountData, BlockChainTrait, BlockProducer, EngineInfo, ImportBlock, MiningBlockChainClient, TermInfo,
};
use crate::consensus::EngineType;
use crate::error::Error;
use crate::transaction::{PendingSignedTransactions, SignedTransaction, UnverifiedTransaction};
use crate::BlockId;
use ckey::{public_to_address, Address, Password, PlatformAddress, Public};
use cstate::{FindActionHandler, TopStateView};
use ctypes::transaction::IncompleteTransaction;
use ctypes::{BlockHash, TxHash};
use cvm::ChainTimeInfo;
use primitives::Bytes;
use std::ops::Range;

/// Miner client API
pub trait MinerService: Send + Sync {
    /// Type representing chain state
    type State: TopStateView + 'static;

    /// Returns miner's status.
    fn status(&self) -> MinerStatus;

    /// Get current authoring parameters.
    fn authoring_params(&self) -> AuthoringParams;

    /// Set the author that we will seal blocks as.
    fn set_author(&self, author: Address) -> Result<(), AccountProviderError>;

    ///Get the address that sealed the block.
    fn get_author_address(&self) -> Address;

    /// Set the extra_data that we will seal blocks with.
    fn set_extra_data(&self, extra_data: Bytes);

    /// Get current transactions limit in queue.
    fn transactions_limit(&self) -> usize;

    /// Set maximal number of transactions kept in the queue (both current and future).
    fn set_transactions_limit(&self, limit: usize);

    /// Called when blocks are imported to chain, updates transactions queue.
    fn chain_new_blocks<C>(
        &self,
        chain: &C,
        imported: &[BlockHash],
        invalid: &[BlockHash],
        enacted: &[BlockHash],
        retracted: &[BlockHash],
    ) where
        C: AccountData + BlockChainTrait + BlockProducer + EngineInfo + ImportBlock;

    /// PoW chain - can produce work package
    fn can_produce_work_package(&self) -> bool;

    /// Get the type of consensus engine.
    fn engine_type(&self) -> EngineType;

    /// Returns true if we had to prepare new pending block.
    fn prepare_work_sealing<C>(&self, _: &C) -> bool
    where
        C: AccountData + BlockChainTrait + BlockProducer + ChainTimeInfo + EngineInfo + FindActionHandler + TermInfo;

    /// New chain head event. Restart mining operation.
    fn update_sealing<C>(&self, chain: &C, parent_block: BlockId, allow_empty_block: bool)
    where
        C: AccountData
            + BlockChainTrait
            + BlockProducer
            + ImportBlock
            + ChainTimeInfo
            + EngineInfo
            + FindActionHandler
            + TermInfo;

    /// Submit `seal` as a valid solution for the header of `pow_hash`.
    /// Will check the seal, but not actually insert the block into the chain.
    fn submit_seal<C: ImportBlock>(&self, chain: &C, pow_hash: BlockHash, seal: Vec<Bytes>) -> Result<(), Error>;

    /// Get the sealing work package and if `Some`, apply some transform.
    fn map_sealing_work<C, F, T>(&self, client: &C, f: F) -> Option<T>
    where
        C: AccountData + BlockChainTrait + BlockProducer + ChainTimeInfo + EngineInfo + FindActionHandler + TermInfo,
        F: FnOnce(&ClosedBlock) -> T,
        Self: Sized;

    /// Imports transactions to mem pool.
    fn import_external_transactions<C: MiningBlockChainClient + EngineInfo + TermInfo>(
        &self,
        client: &C,
        transactions: Vec<UnverifiedTransaction>,
    ) -> Vec<Result<TransactionImportResult, Error>>;

    /// Imports own (node owner) transaction to mem pool.
    fn import_own_transaction<C: MiningBlockChainClient + EngineInfo + TermInfo>(
        &self,
        chain: &C,
        tx: SignedTransaction,
    ) -> Result<TransactionImportResult, Error>;

    /// Imports incomplete (node owner) transaction to mem pool.
    fn import_incomplete_transaction<C: MiningBlockChainClient + AccountData + EngineInfo + TermInfo>(
        &self,
        chain: &C,
        account_provider: &AccountProvider,
        tx: IncompleteTransaction,
        platform_address: PlatformAddress,
        passphrase: Option<Password>,
        seq: Option<u64>,
    ) -> Result<(TxHash, u64), Error>;

    /// Get a list of all pending transactions in the mem pool.
    fn ready_transactions(&self, range: Range<u64>) -> PendingSignedTransactions;

    /// Get list of all future transaction in the mem pool.
    fn future_ready_transactions(&self, range: Range<u64>) -> PendingSignedTransactions;

    /// Get a count of all pending transactions in the mem pool.
    fn count_pending_transactions(&self, range: Range<u64>) -> usize;

    /// a count of all pending transaction including both current and future transactions.
    fn future_included_count_pending_transactions(&self, range: Range<u64>) -> usize;

    /// Get a list of all future transactions.
    fn future_transactions(&self) -> Vec<SignedTransaction>;

    /// Start sealing.
    fn start_sealing<C: MiningBlockChainClient + EngineInfo + TermInfo>(&self, client: &C);

    /// Stop sealing.
    fn stop_sealing(&self);

    /// Get malicious users
    fn get_malicious_users(&self) -> Vec<Address>;

    /// Release target malicious users from malicious user set.
    fn release_malicious_users(&self, prisoner_vec: Vec<Address>);

    /// Imprison target malicious users to malicious user set.
    fn imprison_malicious_users(&self, prisoner_vec: Vec<Address>);

    /// Get ban-immune users.
    fn get_immune_users(&self) -> Vec<Address>;

    /// Register users to ban-immune users.
    fn register_immune_users(&self, immune_user_vec: Vec<Address>);
}

/// Mining status
#[derive(Debug)]
pub struct MinerStatus {
    /// Number of transactions in queue with state `pending` (ready to be included in block)
    pub transactions_in_pending_queue: usize,
    /// Number of transactions in queue with state `future` (not yet ready to be included in block)
    pub transactions_in_future_queue: usize,
    /// Number of transactions included in currently mined block
    pub tranasction_in_pending_block: usize,
}

/// Represents the result of importing tranasction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransactionImportResult {
    /// Tranasction was imported to current queue.
    Current,
    /// Transaction was imported to future queue.
    Future,
}

#[cfg(all(feature = "nightly", test))]
mod mem_pool_benches;

fn fetch_account_creator<'c>(
    client: &'c dyn AccountData,
    block_id: BlockId,
) -> impl Fn(&Public) -> AccountDetails + 'c {
    move |public: &Public| {
        let address = public_to_address(public);
        let a = client.regular_key_owner(&address, block_id.into()).unwrap_or(address);
        AccountDetails {
            seq: client.seq(&a, block_id).expect("We are querying sequence using trusted block id"),
            balance: client.balance(&a, block_id.into()).expect("We are querying balance using trusted block id"),
        }
    }
}
