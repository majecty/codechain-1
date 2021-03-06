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

use cjson::uint::Uint;
use ctypes::transaction::{AssetOutPoint as AssetOutPointType, AssetTransferInput as AssetTransferInputType, Timelock};
use ctypes::{ShardId, Tracker};
use primitives::{Bytes, H160};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetOutPoint {
    pub tracker: Tracker,
    pub index: usize,
    pub asset_type: H160,
    pub shard_id: ShardId,
    pub quantity: Uint,
}

impl From<AssetOutPointType> for AssetOutPoint {
    fn from(from: AssetOutPointType) -> Self {
        AssetOutPoint {
            tracker: from.tracker,
            index: from.index,
            asset_type: from.asset_type,
            shard_id: from.shard_id,
            quantity: from.quantity.into(),
        }
    }
}

impl From<AssetOutPoint> for AssetOutPointType {
    fn from(from: AssetOutPoint) -> Self {
        AssetOutPointType {
            tracker: from.tracker,
            index: from.index,
            asset_type: from.asset_type,
            shard_id: from.shard_id,
            quantity: from.quantity.into(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTransferInput {
    pub prev_out: AssetOutPoint,
    pub timelock: Option<Timelock>,
    pub lock_script: Bytes,
    pub unlock_script: Bytes,
}

impl From<AssetTransferInputType> for AssetTransferInput {
    fn from(from: AssetTransferInputType) -> Self {
        AssetTransferInput {
            prev_out: from.prev_out.into(),
            timelock: from.timelock,
            lock_script: from.lock_script,
            unlock_script: from.unlock_script,
        }
    }
}

impl From<AssetTransferInput> for AssetTransferInputType {
    fn from(from: AssetTransferInput) -> Self {
        AssetTransferInputType {
            prev_out: from.prev_out.into(),
            timelock: from.timelock,
            lock_script: from.lock_script,
            unlock_script: from.unlock_script,
        }
    }
}
