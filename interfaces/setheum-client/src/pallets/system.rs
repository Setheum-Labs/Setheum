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

use primitives::Nonce;
use subxt::utils::Static;

use crate::{
    api, connections::TxInfo, frame_system::pallet::Call::set_code, AccountId, AsConnection,
    Balance, BlockHash, Call::System, ConnectionApi, RootConnection, SudoCall, TxStatus,
};

/// Pallet system read-only api.
#[async_trait::async_trait]
pub trait SystemApi {
    /// returns free balance of a given account
    /// * `account` - account id
    /// * `at` - optional hash of a block to query state from
    ///
    /// it uses [`system.account`](https://paritytech.github.io/substrate/master/frame_system/pallet/struct.Pallet.html#method.account) storage
    async fn get_free_balance(&self, account: AccountId, at: Option<BlockHash>) -> Balance;

    /// returns account nonce of a given account
    /// * `account` - account id
    async fn account_nonce(&self, account: &AccountId) -> anyhow::Result<Nonce>;
}

/// Pallet system api.
#[async_trait::async_trait]
pub trait SystemSudoApi {
    /// API for [`set_code`](https://paritytech.github.io/substrate/master/frame_system/pallet/struct.Pallet.html#method.set_code) call.
    async fn set_code(&self, code: Vec<u8>, status: TxStatus) -> anyhow::Result<TxInfo>;
}

#[async_trait::async_trait]
impl SystemSudoApi for RootConnection {
    async fn set_code(&self, code: Vec<u8>, status: TxStatus) -> anyhow::Result<TxInfo> {
        let call = System(set_code { code });

        self.sudo_unchecked(call, status).await
    }
}

#[async_trait::async_trait]
impl<C: AsConnection + Sync> SystemApi for C {
    async fn get_free_balance(&self, account: AccountId, at: Option<BlockHash>) -> Balance {
        let addrs = api::storage().system().account(Static(account));

        match self.get_storage_entry_maybe(&addrs, at).await {
            None => 0,
            Some(account) => account.data.free,
        }
    }

    async fn account_nonce(&self, account: &AccountId) -> anyhow::Result<Nonce> {
        let conn = self.as_connection();
        Ok(conn.client.tx().account_nonce(account).await?.try_into()?)
    }
}
