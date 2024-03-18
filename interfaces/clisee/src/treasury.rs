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
    pallets::treasury::{TreasurySudoApi, TreasuryUserApi},
    AccountId, RootConnection, SignedConnection, Ss58Codec, TxStatus,
};
use primitives::{Balance, TOKEN};

/// Delegates to `setheum_client::make_treasury_proposal`.
pub async fn propose(connection: SignedConnection, amount_in_tokens: u64, beneficiary: String) {
    let beneficiary = AccountId::from_ss58check(&beneficiary).expect("Address should be valid");
    let endowment = amount_in_tokens as Balance * TOKEN;

    connection
        .propose_spend(endowment, beneficiary, TxStatus::Finalized)
        .await
        .unwrap();
}

/// Delegates to `setheum_client::approve_treasury_proposal`.
pub async fn approve(connection: RootConnection, proposal_id: u32) {
    TreasurySudoApi::approve(&connection, proposal_id, TxStatus::Finalized)
        .await
        .unwrap();
}

/// Delegates to `setheum_client::reject_treasury_proposal`.
pub async fn reject(connection: RootConnection, proposal_id: u32) {
    TreasurySudoApi::reject(&connection, proposal_id, TxStatus::Finalized)
        .await
        .unwrap();
}
