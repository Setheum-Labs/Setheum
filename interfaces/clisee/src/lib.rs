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

mod commands;
mod contracts;
mod finalization;
mod keys;
mod runtime;
mod secret;
mod staking;
mod transfer;
mod treasury;
mod validators;
mod version_upgrade;
mod vesting;
mod vk_storage;

use setheum_client::{keypair_from_string, Connection, RootConnection, SignedConnection};
pub use commands::{Command, VkStorage};
pub use contracts::{
    call, code_info, instantiate, instantiate_with_code, remove_code, upload_code,
};
pub use finalization::{finalize, set_emergency_finalizer};
pub use keys::{next_session_keys, prepare_keys, rotate_keys, set_keys};
pub use runtime::update_runtime;
pub use secret::prompt_password_hidden;
pub use staking::{bond, force_new_era, nominate, set_staking_limits, validate};
pub use transfer::transfer_keep_alive;
pub use treasury::{
    approve as treasury_approve, propose as treasury_propose, reject as treasury_reject,
};
pub use validators::change_validators;
pub use version_upgrade::schedule_upgrade;
pub use vesting::{vest, vest_other, vested_transfer};
pub use vk_storage::store_key;

pub struct ConnectionConfig {
    node_endpoint: String,
    signer_seed: String,
}

impl ConnectionConfig {
    pub fn new(node_endpoint: String, signer_seed: String) -> Self {
        ConnectionConfig {
            node_endpoint,
            signer_seed,
        }
    }

    pub async fn get_connection(&self) -> Connection {
        Connection::new(&self.node_endpoint).await
    }

    pub async fn get_signed_connection(&self) -> SignedConnection {
        SignedConnection::new(&self.node_endpoint, keypair_from_string(&self.signer_seed)).await
    }

    pub async fn get_root_connection(&self) -> RootConnection {
        RootConnection::new(&self.node_endpoint, keypair_from_string(&self.signer_seed))
            .await
            .expect("signer should be root")
    }
}
