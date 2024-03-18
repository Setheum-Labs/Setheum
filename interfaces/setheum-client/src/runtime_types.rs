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

pub use crate::setheum::api::runtime_types::*;
use crate::{
    setheum_runtime::SessionKeys,
    api::runtime_types::{
        primitives::app::Public as AlephPublic,
        sp_consensus_aura::sr25519::app_sr25519::Public as AuraPublic,
        sp_core::{ed25519::Public as EdPublic, sr25519::Public as SrPublic},
    },
    pallet_staking::EraRewardPoints,
    sp_weights::weight_v2::Weight,
};

impl<AccountId> Default for EraRewardPoints<AccountId> {
    fn default() -> Self {
        Self {
            total: 0,
            individual: vec![],
        }
    }
}

// Manually implementing decoding
impl From<Vec<u8>> for SessionKeys {
    fn from(bytes: Vec<u8>) -> Self {
        assert_eq!(bytes.len(), 64);
        Self {
            aura: AuraPublic(SrPublic(
                bytes[..32]
                    .try_into()
                    .expect("Failed to convert bytes slice to an Aura key!"),
            )),
            aleph: AlephPublic(EdPublic(
                bytes[32..64]
                    .try_into()
                    .expect("Failed to convert bytes slice to an Aleph key!"),
            )),
        }
    }
}

impl TryFrom<String> for SessionKeys {
    type Error = ();

    fn try_from(keys: String) -> Result<Self, Self::Error> {
        let bytes: Vec<u8> = match hex::FromHex::from_hex(keys) {
            Ok(bytes) => bytes,
            Err(_) => return Err(()),
        };
        Ok(SessionKeys::from(bytes))
    }
}

impl Weight {
    /// Returns new instance of weight v2 object.
    pub const fn new(ref_time: u64, proof_size: u64) -> Self {
        Self {
            ref_time,
            proof_size,
        }
    }
}
