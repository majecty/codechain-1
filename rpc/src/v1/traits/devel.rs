// Copyright 2018-2020 Kodebox, Inc.
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

use super::super::types::TPSTestSetting;
use cjson::bytes::Bytes;
use ctypes::BlockHash;
use jsonrpc_core::Result;
use primitives::H256;
use std::net::SocketAddr;

#[rpc(server)]
pub trait Devel {
    #[rpc(name = "devel_getStateTrieKeys")]
    fn get_state_trie_keys(&self, offset: usize, limit: usize) -> Result<Vec<H256>>;

    #[rpc(name = "devel_getStateTrieValue")]
    fn get_state_trie_value(&self, key: H256) -> Result<Vec<Bytes>>;

    #[rpc(name = "devel_startSealing")]
    fn start_sealing(&self) -> Result<()>;

    #[rpc(name = "devel_stopSealing")]
    fn stop_sealing(&self) -> Result<()>;

    #[rpc(name = "devel_getBlockSyncPeers")]
    fn get_block_sync_peers(&self) -> Result<Vec<SocketAddr>>;

    #[rpc(name = "devel_getPeerBestBlockHashes")]
    fn get_peer_best_block_hashes(&self) -> Result<Vec<(SocketAddr, BlockHash)>>;

    #[rpc(name = "devel_getTargetBlockHashes")]
    fn get_target_block_hashes(&self) -> Result<Vec<BlockHash>>;

    #[rpc(name = "devel_testTPS")]
    fn test_tps(&self, setting: TPSTestSetting) -> Result<f64>;
}
