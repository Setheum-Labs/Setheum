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

use setheum_runtime_interfaces::snark_verifier::{verify, VerifierError};
use module_vk_storage::{Config as VkStorageConfig, VerificationKeys};

use crate::args::VerifyArgs;

/// Represents an 'engine' that handles chain extension calls.
pub trait BackendExecutor {
    fn verify(args: VerifyArgs) -> Result<(), VerifierError>;
}

/// Default implementation for the chain extension mechanics.
impl<Runtime: VkStorageConfig> BackendExecutor for Runtime {
    fn verify(args: VerifyArgs) -> Result<(), VerifierError> {
        let verifying_key = VerificationKeys::<Runtime>::get(args.verification_key_hash)
            .ok_or(VerifierError::UnknownVerificationKeyIdentifier)?
            .to_vec();

        verify(&args.proof, &args.public_input, &verifying_key)
    }
}
