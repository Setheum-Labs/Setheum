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

use setheum_client::{
    pallets::staking::{StakingSudoApi, StakingUserApi},
    AccountId, Balance, RootConnection, SignedConnection, Ss58Codec, TxStatus,
};
use primitives::TOKEN;

pub async fn bond(stash_connection: SignedConnection, initial_stake_in_tokens: u32) {
    let initial_stake = initial_stake_in_tokens as Balance * TOKEN;
    stash_connection
        .bond(initial_stake, TxStatus::Finalized)
        .await
        .unwrap();
}

pub async fn validate(connection: SignedConnection, commission_percentage: u8) {
    connection
        .validate(commission_percentage, TxStatus::Finalized)
        .await
        .unwrap();
}

pub async fn nominate(connection: SignedConnection, nominee: String) {
    let nominee_account = AccountId::from_ss58check(&nominee).expect("Address is valid");
    connection
        .nominate(nominee_account, TxStatus::InBlock)
        .await
        .unwrap();
}

pub async fn set_staking_limits(
    root_connection: RootConnection,
    minimal_nominator_stake_tokens: u64,
    minimal_validator_stake_tokens: u64,
    max_nominators_count: Option<u32>,
    max_validators_count: Option<u32>,
) {
    root_connection
        .set_staking_config(
            Some(minimal_nominator_stake_tokens as Balance * TOKEN),
            Some(minimal_validator_stake_tokens as Balance * TOKEN),
            max_nominators_count,
            max_validators_count,
            TxStatus::Finalized,
        )
        .await
        .unwrap();
}

pub async fn force_new_era(root_connection: RootConnection) {
    root_connection
        .force_new_era(TxStatus::Finalized)
        .await
        .unwrap();
}
