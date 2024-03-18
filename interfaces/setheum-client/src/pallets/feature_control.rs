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

use crate::{
    api, module_feature_control::Feature, BlockHash, ConnectionApi, RootConnection,
    SignedConnectionApi, TxInfo, TxStatus,
};

/// Read only pallet feature control API.
#[async_trait::async_trait]
pub trait FeatureControlApi {
    /// Check if a feature is active.
    async fn is_feature_active(&self, feature: Feature, at: Option<BlockHash>) -> bool;
}

/// Pallet feature control API that requires sudo.
#[async_trait::async_trait]
pub trait FeatureControlSudoApi {
    /// Enable a feature.
    async fn enable_feature(&self, feature: Feature, status: TxStatus) -> anyhow::Result<TxInfo>;
    /// Disable a feature.
    async fn disable_feature(&self, feature: Feature, status: TxStatus) -> anyhow::Result<TxInfo>;
}

#[async_trait::async_trait]
impl<C: ConnectionApi> FeatureControlApi for C {
    async fn is_feature_active(&self, feature: Feature, at: Option<BlockHash>) -> bool {
        let addrs = api::storage().feature_control().active_features(feature);
        self.get_storage_entry_maybe(&addrs, at).await.is_some()
    }
}

#[async_trait::async_trait]
impl FeatureControlSudoApi for RootConnection {
    async fn enable_feature(&self, feature: Feature, status: TxStatus) -> anyhow::Result<TxInfo> {
        let tx = api::tx().feature_control().enable(feature);
        self.send_tx(tx, status).await
    }

    async fn disable_feature(&self, feature: Feature, status: TxStatus) -> anyhow::Result<TxInfo> {
        let tx = api::tx().feature_control().disable(feature);
        self.send_tx(tx, status).await
    }
}
