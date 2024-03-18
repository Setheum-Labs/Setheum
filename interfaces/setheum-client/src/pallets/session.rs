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

use subxt::utils::Static;

use crate::{
    api, api::runtime_types::setheum_runtime::SessionKeys, connections::TxInfo, AccountId, BlockHash,
    ConnectionApi, SessionIndex, SignedConnectionApi, TxStatus,
};

/// Pallet session read-only api.
#[async_trait::async_trait]
pub trait SessionApi {
    /// API for [`next_keys`](https://paritytech.github.io/substrate/master/pallet_session/pallet/type.NextKeys.html) call.
    async fn get_next_session_keys(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Option<SessionKeys>;

    /// API for [`current_index`](https://paritytech.github.io/substrate/master/pallet_session/pallet/struct.Pallet.html#method.current_index) call.
    async fn get_session(&self, at: Option<BlockHash>) -> SessionIndex;

    /// API for [`validators`](https://paritytech.github.io/substrate/master/pallet_session/pallet/struct.Pallet.html#method.validators) call.
    async fn get_validators(&self, at: Option<BlockHash>) -> Vec<AccountId>;
}

/// any object that implements pallet session api
#[async_trait::async_trait]
pub trait SessionUserApi {
    /// API for [`set_keys`](https://paritytech.github.io/substrate/master/pallet_session/pallet/struct.Pallet.html#method.set_keys) call.
    async fn set_keys(&self, new_keys: SessionKeys, status: TxStatus) -> anyhow::Result<TxInfo>;
}

#[async_trait::async_trait]
impl<C: ConnectionApi> SessionApi for C {
    async fn get_next_session_keys(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Option<SessionKeys> {
        let addrs = api::storage().session().next_keys(Static::from(account));

        self.get_storage_entry_maybe(&addrs, at).await
    }

    async fn get_session(&self, at: Option<BlockHash>) -> SessionIndex {
        let addrs = api::storage().session().current_index();

        self.get_storage_entry_maybe(&addrs, at)
            .await
            .unwrap_or_default()
    }

    async fn get_validators(&self, at: Option<BlockHash>) -> Vec<AccountId> {
        let addrs = api::storage().session().validators();

        self.get_storage_entry(&addrs, at)
            .await
            .into_iter()
            .map(|x| x.0)
            .collect()
    }
}

#[async_trait::async_trait]
impl<S: SignedConnectionApi> SessionUserApi for S {
    async fn set_keys(&self, new_keys: SessionKeys, status: TxStatus) -> anyhow::Result<TxInfo> {
        let tx = api::tx().session().set_keys(new_keys, vec![]);

        self.send_tx(tx, status).await
    }
}
