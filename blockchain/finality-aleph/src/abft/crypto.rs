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

use crate::{
    crypto::{AuthorityPen, AuthorityVerifier, Signature},
    NodeCount, NodeIndex, SignatureSet,
};

/// Keychain combines an AuthorityPen and AuthorityVerifier into one object implementing the AlephBFT
/// MultiKeychain trait.
#[derive(Clone)]
pub struct Keychain {
    id: NodeIndex,
    authority_pen: AuthorityPen,
    authority_verifier: AuthorityVerifier,
}

impl Keychain {
    /// Constructs a new keychain from a signing contraption and verifier, with the specified node
    /// index.
    pub fn new(
        id: NodeIndex,
        authority_verifier: AuthorityVerifier,
        authority_pen: AuthorityPen,
    ) -> Self {
        Keychain {
            id,
            authority_pen,
            authority_verifier,
        }
    }

    fn index(&self) -> NodeIndex {
        self.id
    }

    fn node_count(&self) -> NodeCount {
        self.authority_verifier.node_count()
    }

    fn sign(&self, msg: &[u8]) -> Signature {
        self.authority_pen.sign(msg)
    }

    fn verify<I: Into<NodeIndex>>(&self, msg: &[u8], sgn: &Signature, index: I) -> bool {
        self.authority_verifier.verify(msg, sgn, index.into())
    }

    fn is_complete(&self, msg: &[u8], partial: &SignatureSet<Signature>) -> bool {
        self.authority_verifier.is_complete(msg, partial)
    }
}

impl current_aleph_bft::Index for Keychain {
    fn index(&self) -> current_aleph_bft::NodeIndex {
        Keychain::index(self).into()
    }
}

impl legacy_aleph_bft::Index for Keychain {
    fn index(&self) -> legacy_aleph_bft::NodeIndex {
        Keychain::index(self).into()
    }
}

#[async_trait::async_trait]
impl current_aleph_bft::Keychain for Keychain {
    type Signature = Signature;

    fn node_count(&self) -> current_aleph_bft::NodeCount {
        Keychain::node_count(self).into()
    }

    fn sign(&self, msg: &[u8]) -> Signature {
        Keychain::sign(self, msg)
    }

    fn verify(&self, msg: &[u8], sgn: &Signature, index: current_aleph_bft::NodeIndex) -> bool {
        Keychain::verify(self, msg, sgn, index)
    }
}

#[async_trait::async_trait]
impl legacy_aleph_bft::Keychain for Keychain {
    type Signature = Signature;

    fn node_count(&self) -> legacy_aleph_bft::NodeCount {
        Keychain::node_count(self).into()
    }

    async fn sign(&self, msg: &[u8]) -> Signature {
        Keychain::sign(self, msg)
    }

    fn verify(&self, msg: &[u8], sgn: &Signature, index: legacy_aleph_bft::NodeIndex) -> bool {
        Keychain::verify(self, msg, sgn, index)
    }
}

impl current_aleph_bft::MultiKeychain for Keychain {
    // Using `SignatureSet` is slow, but Substrate has not yet implemented aggregation.
    // We probably should do this for them at some point.
    type PartialMultisignature = SignatureSet<Signature>;

    fn bootstrap_multi(
        &self,
        signature: &Signature,
        index: current_aleph_bft::NodeIndex,
    ) -> Self::PartialMultisignature {
        current_aleph_bft::PartialMultisignature::add_signature(
            SignatureSet(aleph_bft_crypto::SignatureSet::with_size(
                aleph_bft_crypto::Keychain::node_count(self),
            )),
            signature,
            index,
        )
    }

    fn is_complete(&self, msg: &[u8], partial: &Self::PartialMultisignature) -> bool {
        Keychain::is_complete(self, msg, partial)
    }
}

impl legacy_aleph_bft::MultiKeychain for Keychain {
    // Using `SignatureSet` is slow, but Substrate has not yet implemented aggregation.
    // We probably should do this for them at some point.
    type PartialMultisignature = SignatureSet<Signature>;

    fn bootstrap_multi(
        &self,
        signature: &Signature,
        index: legacy_aleph_bft::NodeIndex,
    ) -> Self::PartialMultisignature {
        legacy_aleph_bft::PartialMultisignature::add_signature(
            SignatureSet(aleph_bft_crypto::SignatureSet::with_size(
                aleph_bft_crypto::Keychain::node_count(self),
            )),
            signature,
            index,
        )
    }

    fn is_complete(&self, msg: &[u8], partial: &Self::PartialMultisignature) -> bool {
        Keychain::is_complete(self, msg, partial)
    }
}
