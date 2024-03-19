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

//! Module gathering all the chain extension arguments. They can be used in the smart contract for a
//! proper argument encoding. On the runtime side, they can be used for decoding the arguments.

#[cfg(feature = "ink")]
use ink::prelude::vec::Vec;
#[cfg(feature = "runtime")]
use {
    parity_scale_codec::{Decode, Encode},
    sp_std::vec::Vec,
};

/// A struct describing layout for the `verify` chain extension.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ink", ink::scale_derive(Encode, Decode))]
#[cfg_attr(feature = "runtime", derive(Encode, Decode))]
pub struct VerifyArgs {
    /// The hash of the verification key.
    pub verification_key_hash: crate::KeyHash,
    /// The proof.
    pub proof: Vec<u8>,
    /// The public input.
    pub public_input: Vec<u8>,
}
