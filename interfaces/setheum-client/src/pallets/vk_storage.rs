// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of Setheum.

// Copyright (C) 2019-Present Setheum Labs.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use anyhow::Result;

use crate::{
    api, sp_core::H256, BlockHash, ConnectionApi, SignedConnection, SignedConnectionApi, TxInfo,
    TxStatus,
};

/// Read only pallet vk storage API.
#[async_trait::async_trait]
pub trait VkStorageApi {
    /// Get verification key from pallet's storage.
    async fn get_verification_key(&self, key_hash: H256, at: Option<BlockHash>) -> Vec<u8>;
}

/// Pallet vk storage API.
#[async_trait::async_trait]
pub trait VkStorageUserApi {
    /// Store a verifying key in pallet's storage.
    async fn store_key(&self, key: Vec<u8>, status: TxStatus) -> Result<TxInfo>;
}

#[async_trait::async_trait]
impl<C: ConnectionApi> VkStorageApi for C {
    async fn get_verification_key(&self, key_hash: H256, at: Option<BlockHash>) -> Vec<u8> {
        let addrs = api::storage().vk_storage().verification_keys(key_hash);
        self.get_storage_entry(&addrs, at).await.0
    }
}

#[async_trait::async_trait]
impl VkStorageUserApi for SignedConnection {
    async fn store_key(&self, key: Vec<u8>, status: TxStatus) -> Result<TxInfo> {
        let tx = api::tx().vk_storage().store_key(key);
        self.send_tx(tx, status).await
    }
}
