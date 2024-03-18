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
    account_from_keypair, keypair_from_string, module_vesting::vesting_info::VestingInfo,
    pallets::vesting::VestingUserApi, SignedConnection, TxStatus,
};
use log::{error, info};
use primitives::{Balance, BlockNumber, TOKEN};

/// Delegates to `setheum_client::vest`.
///
/// Vesting is performed for the signer of `connection`.
pub async fn vest(connection: SignedConnection) {
    match connection.vest(TxStatus::Finalized).await {
        Ok(_) => info!("Vesting has succeeded"),
        Err(e) => error!("Vesting has failed with:\n {:?}", e),
    }
}

/// Delegates to `setheum_client::vest_other`.
///
/// Vesting is performed by the signer of `connection` for `vesting_account_seed`.
pub async fn vest_other(connection: SignedConnection, vesting_account_seed: String) {
    let vester = account_from_keypair(keypair_from_string(vesting_account_seed.as_str()).signer());
    match connection.vest_other(TxStatus::Finalized, vester).await {
        Ok(_) => info!("Vesting on behalf has succeeded"),
        Err(e) => error!("Vesting on behalf has failed with:\n {:?}", e),
    }
}

/// Delegates to `setheum_client::vested_transfer`.
///
/// The transfer is performed from the signer of `connection` to `target_seed`.
/// `amount_in_tokens`, `per_block` and `starting_block` corresponds to the fields of
/// `setheum_client::VestingSchedule` struct.
pub async fn vested_transfer(
    connection: SignedConnection,
    target_seed: String,
    amount_in_tokens: u64,
    per_block: Balance,
    starting_block: BlockNumber,
) {
    let receiver = account_from_keypair(keypair_from_string(target_seed.as_str()).signer());
    let schedule = VestingInfo {
        locked: amount_in_tokens as Balance * TOKEN,
        per_block,
        starting_block,
    };
    match connection
        .vested_transfer(receiver, schedule, TxStatus::Finalized)
        .await
    {
        Ok(_) => info!("Vested transfer has succeeded"),
        Err(e) => error!("Vested transfer has failed with:\n {:?}", e),
    }
}
