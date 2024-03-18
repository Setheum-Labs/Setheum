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

use std::str::FromStr;

use setheum_client::{
    pallets::aleph::{AlephRpc, AlephSudoApi},
    AccountId, AlephKeyPair, Connection, TxStatus,
};
use primitives::{BlockHash, BlockNumber};

use crate::RootConnection;

/// Sets the emergency finalized, the provided string should be the seed phrase of the desired finalizer.
pub async fn set_emergency_finalizer(connection: RootConnection, finalizer: AccountId) {
    connection
        .set_emergency_finalizer(finalizer, TxStatus::Finalized)
        .await
        .unwrap();
}

/// Finalizes the given block using the key pair from provided seed as emergency finalizer.
pub async fn finalize(
    connection: Connection,
    number: BlockNumber,
    hash: String,
    key_pair: AlephKeyPair,
) {
    let hash = BlockHash::from_str(&hash).expect("Hash is properly hex encoded");
    connection
        .emergency_finalize(number, hash, key_pair)
        .await
        .unwrap();
}
