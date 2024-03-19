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

//! This is the frontend of the chain extension, i.e., the part exposed to the smart contracts.

use ink::{
    env::{DefaultEnvironment, Environment as EnvironmentT},
    prelude::vec::Vec,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[allow(missing_docs)] // Error variants are self-descriptive.
/// Chain extension errors enumeration.
pub enum BlackBoxError {
    // Proof verification errors.
    UnknownVerificationKeyIdentifier,
    DeserializingPublicInputFailed,
    DeserializingVerificationKeyFailed,
    VerificationFailed,
    IncorrectProof,
    VerifyErrorUnknown,

    /// Couldn't serialize or deserialize data.
    ScaleError,
    /// Unexpected error code has been returned.
    UnknownError(u32),
}

impl From<ink::scale::Error> for BlackBoxError {
    fn from(_: ink::scale::Error) -> Self {
        Self::ScaleError
    }
}

impl ink::env::chain_extension::FromStatusCode for BlackBoxError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        use crate::status_codes::*;

        match status_code {
            // Success codes
            VERIFY_SUCCESS => Ok(()),

            // Proof verification errors
            VERIFY_DESERIALIZING_INPUT_FAIL => Err(Self::DeserializingPublicInputFailed),
            VERIFY_UNKNOWN_IDENTIFIER => Err(Self::UnknownVerificationKeyIdentifier),
            VERIFY_DESERIALIZING_KEY_FAIL => Err(Self::DeserializingVerificationKeyFailed),
            VERIFY_VERIFICATION_FAIL => Err(Self::VerificationFailed),
            VERIFY_INCORRECT_PROOF => Err(Self::IncorrectProof),

            unexpected => Err(Self::UnknownError(unexpected)),
        }
    }
}

/// BlackBox chain extension definition.
// IMPORTANT: this must match the extension ID in `extension_ids.rs`! However, because constants are not inlined before
// macro processing, we can't use an identifier from another module here.
#[ink::chain_extension(extension = 41)]
pub trait BlackBoxExtension {
    type ErrorCode = BlackBoxError;

    /// Verify a ZK proof `proof` given the public input `input` against the verification key
    /// `identifier`.
    // IMPORTANT: this must match the function ID in `extension_ids.rs`! However, because constants are not inlined
    // before macro processing, we can't use an identifier from another module here.
    #[ink(function = 0)]
    fn verify(
        identifier: crate::KeyHash,
        proof: Vec<u8>,
        input: Vec<u8>,
    ) -> Result<(), BlackBoxError>;
}

/// Default ink environment with `BlackBoxExtension` included.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Environment {}

impl EnvironmentT for Environment {
    const MAX_EVENT_TOPICS: usize = <DefaultEnvironment as EnvironmentT>::MAX_EVENT_TOPICS;

    type AccountId = <DefaultEnvironment as EnvironmentT>::AccountId;
    type Balance = <DefaultEnvironment as EnvironmentT>::Balance;
    type Hash = <DefaultEnvironment as EnvironmentT>::Hash;
    type BlockNumber = <DefaultEnvironment as EnvironmentT>::BlockNumber;
    type Timestamp = <DefaultEnvironment as EnvironmentT>::Timestamp;

    type ChainExtension = BlackBoxExtension;
}
