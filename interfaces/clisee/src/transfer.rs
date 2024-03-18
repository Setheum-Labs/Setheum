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
    pallets::balances::BalanceUserApi, AccountId, Balance, SignedConnection, Ss58Codec, TxStatus,
};
use primitives::TOKEN;

pub async fn transfer_keep_alive(
    connection: SignedConnection,
    amount_in_tokens: u64,
    to_account: String,
) {
    let to_account = AccountId::from_ss58check(&to_account).expect("Address is valid");
    connection
        .transfer_keep_alive(
            to_account,
            amount_in_tokens as Balance * TOKEN,
            TxStatus::Finalized,
        )
        .await
        .unwrap();
}
