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

use std::{fmt::Display, hash::Hash};

use parity_scale_codec::Codec;

/// A public key for signature verification.
pub trait PublicKey:
    Send + Sync + Eq + Clone + AsRef<[u8]> + Display + Hash + Codec + 'static
{
    type Signature: Send + Sync + Clone + Codec;

    /// Verify whether the message has been signed with the associated private key.
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool;
}

/// Secret key for signing messages, with an associated public key.
pub trait SecretKey: Clone + Send + Sync + 'static {
    type Signature: Send + Sync + Clone + Codec;
    type PublicKey: PublicKey<Signature = Self::Signature>;

    /// Produce a signature for the provided message.
    fn sign(&self, message: &[u8]) -> Self::Signature;

    /// Return the associated public key.
    fn public_key(&self) -> Self::PublicKey;
}
