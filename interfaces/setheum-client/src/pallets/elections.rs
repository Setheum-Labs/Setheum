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
    api,
    api::runtime_types::primitives::{CommitteeSeats, EraValidators},
    connections::{AsConnection, TxInfo},
    module_elections::pallet::Call::{change_validators, set_elections_openness},
    primitives::ElectionOpenness,
    AccountId, BlockHash,
    Call::Elections,
    ConnectionApi, RootConnection, SudoCall, TxStatus,
};

// TODO once pallet elections docs are published, replace api docs with links to public docs
/// Pallet elections read-only api.
#[async_trait::async_trait]
pub trait ElectionsApi {
    /// Returns `elections.committee_size` storage of the elections pallet.
    /// * `at` - optional hash of a block to query state from
    async fn get_committee_seats(&self, at: Option<BlockHash>) -> CommitteeSeats;

    /// Returns `elections.next_era_committee_seats` storage of the elections pallet.
    /// * `at` - optional hash of a block to query state from
    async fn get_next_era_committee_seats(&self, at: Option<BlockHash>) -> CommitteeSeats;

    /// Returns `elections.current_era_validators` storage of the elections pallet.
    /// * `at` - optional hash of a block to query state from
    async fn get_current_era_validators(&self, at: Option<BlockHash>) -> EraValidators<AccountId>;

    /// Returns `elections.next_era_reserved_validators` storage of the elections pallet.
    /// * `at` - optional hash of a block to query state from
    async fn get_next_era_reserved_validators(&self, at: Option<BlockHash>) -> Vec<AccountId>;

    /// Returns `elections.next_era_non_reserved_validators` storage of the elections pallet.
    /// * `at` - optional hash of a block to query state from
    async fn get_next_era_non_reserved_validators(&self, at: Option<BlockHash>) -> Vec<AccountId>;
}

/// any object that implements pallet elections api that requires sudo
#[async_trait::async_trait]
pub trait ElectionsSudoApi {
    /// Issues `elections.change_validators` that sets the committee for the next era.
    /// * `new_reserved_validators` - reserved validators to be in place in the next era; optional
    /// * `new_non_reserved_validators` - non reserved validators to be in place in the next era; optional
    /// * `committee_size` - committee size to be in place in the next era; optional
    /// * `status` - a [`TxStatus`] for a tx to wait for
    async fn change_validators(
        &self,
        new_reserved_validators: Option<Vec<AccountId>>,
        new_non_reserved_validators: Option<Vec<AccountId>>,
        committee_size: Option<CommitteeSeats>,
        status: TxStatus,
    ) -> anyhow::Result<TxInfo>;

    /// Set openness of the elections.
    /// * `mode` - new elections openness mode
    /// * `status` - a [`TxStatus`] for a tx to wait for
    async fn set_election_openness(
        &self,
        mode: ElectionOpenness,
        status: TxStatus,
    ) -> anyhow::Result<TxInfo>;
}

#[async_trait::async_trait]
impl<C: ConnectionApi + AsConnection> ElectionsApi for C {
    async fn get_committee_seats(&self, at: Option<BlockHash>) -> CommitteeSeats {
        let addrs = api::storage().elections().committee_size();

        self.get_storage_entry(&addrs, at).await
    }

    async fn get_next_era_committee_seats(&self, at: Option<BlockHash>) -> CommitteeSeats {
        let addrs = api::storage().elections().next_era_committee_size();

        self.get_storage_entry(&addrs, at).await
    }

    async fn get_current_era_validators(&self, at: Option<BlockHash>) -> EraValidators<AccountId> {
        let addrs = api::storage().elections().current_era_validators();
        let era_validators_with_static_account_ids = self.get_storage_entry(&addrs, at).await;
        return EraValidators {
            reserved: era_validators_with_static_account_ids
                .reserved
                .into_iter()
                .map(|x| x.0)
                .collect(),
            non_reserved: era_validators_with_static_account_ids
                .non_reserved
                .into_iter()
                .map(|x| x.0)
                .collect(),
        };
    }

    async fn get_next_era_reserved_validators(&self, at: Option<BlockHash>) -> Vec<AccountId> {
        let addrs = api::storage().elections().next_era_reserved_validators();

        self.get_storage_entry(&addrs, at)
            .await
            .into_iter()
            .map(|x| x.0)
            .collect()
    }

    async fn get_next_era_non_reserved_validators(&self, at: Option<BlockHash>) -> Vec<AccountId> {
        let addrs = api::storage()
            .elections()
            .next_era_non_reserved_validators();

        self.get_storage_entry(&addrs, at)
            .await
            .into_iter()
            .map(|x| x.0)
            .collect()
    }
}

#[async_trait::async_trait]
impl ElectionsSudoApi for RootConnection {
    async fn change_validators(
        &self,
        new_reserved_validators: Option<Vec<AccountId>>,
        new_non_reserved_validators: Option<Vec<AccountId>>,
        committee_size: Option<CommitteeSeats>,
        status: TxStatus,
    ) -> anyhow::Result<TxInfo> {
        let call = Elections(change_validators {
            reserved_validators: new_reserved_validators
                .map(|inner| inner.into_iter().map(Static).collect()),
            non_reserved_validators: new_non_reserved_validators
                .map(|inner| inner.into_iter().map(Static).collect()),
            committee_size,
        });

        self.sudo_unchecked(call, status).await
    }

    async fn set_election_openness(
        &self,
        mode: ElectionOpenness,
        status: TxStatus,
    ) -> anyhow::Result<TxInfo> {
        let call = Elections(set_elections_openness { openness: mode });

        self.sudo_unchecked(call, status).await
    }
}
