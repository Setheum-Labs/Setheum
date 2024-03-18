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

use subxt::utils::MultiAddress;

use crate::{
    setheum_runtime::{ProxyType, RuntimeCall},
    api, AccountId, SignedConnectionApi, TxInfo, TxStatus,
};

/// any object that implements pallet proxy api
#[async_trait::async_trait]
pub trait ProxyUserApi {
    /// API for [`proxy`](https://paritytech.github.io/polkadot-sdk/master/pallet_proxy/pallet/struct.Pallet.html#method.proxy) call.
    async fn proxy(
        &self,
        real: AccountId,
        call: RuntimeCall,
        status: TxStatus,
    ) -> anyhow::Result<TxInfo>;

    /// API for [`add_proxy`](https://paritytech.github.io/polkadot-sdk/master/pallet_proxy/pallet/struct.Pallet.html#method.add_proxy) call.
    async fn add_proxy(
        &self,
        delegate: AccountId,
        proxy_type: ProxyType,
        delay: u32,
        status: TxStatus,
    ) -> anyhow::Result<TxInfo>;

    /// API for [`remove_proxy`](https://paritytech.github.io/polkadot-sdk/master/pallet_proxy/pallet/struct.Pallet.html#method.remove_proxy) call.
    async fn remove_proxy(
        &self,
        delegate: AccountId,
        proxy_type: ProxyType,
        delay: u32,
        status: TxStatus,
    ) -> anyhow::Result<TxInfo>;
}

#[async_trait::async_trait]
impl<S: SignedConnectionApi> ProxyUserApi for S {
    async fn proxy(
        &self,
        real: AccountId,
        call: RuntimeCall,
        status: TxStatus,
    ) -> anyhow::Result<TxInfo> {
        let tx = api::tx()
            .proxy()
            .proxy(MultiAddress::Id(real.into()), None, call);

        self.send_tx(tx, status).await
    }
    async fn add_proxy(
        &self,
        delegate: AccountId,
        proxy_type: ProxyType,
        delay: u32,
        status: TxStatus,
    ) -> anyhow::Result<TxInfo> {
        let tx = api::tx()
            .proxy()
            .add_proxy(MultiAddress::Id(delegate.into()), proxy_type, delay);

        self.send_tx(tx, status).await
    }
    async fn remove_proxy(
        &self,
        delegate: AccountId,
        proxy_type: ProxyType,
        delay: u32,
        status: TxStatus,
    ) -> anyhow::Result<TxInfo> {
        let tx =
            api::tx()
                .proxy()
                .remove_proxy(MultiAddress::Id(delegate.into()), proxy_type, delay);

        self.send_tx(tx, status).await
    }
}
