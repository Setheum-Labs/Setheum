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

//! # Senterprise Extension
//!
//! This crate provides a way for smart contracts to work with ZK proofs (SNARKs).

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
// For testing purposes, we need to enable some unstable features.
#![cfg_attr(test, allow(incomplete_features))]
#![cfg_attr(test, feature(adt_const_params))]
#![cfg_attr(test, feature(generic_const_exprs))]

// Rust features are additive, so this is the only way we can ensure that only one of these is
// enabled.
#[cfg(all(feature = "ink", feature = "runtime"))]
compile_error!("Features `ink` and `runtime` are mutually exclusive and cannot be used together");

#[cfg(not(any(feature = "ink", feature = "runtime")))]
compile_error!("Either `ink` or `runtime` feature must be enabled (or their `-std` extensions)");

// ------ Common stuff -----------------------------------------------------------------------------

pub mod args;
pub mod extension_ids;
pub mod status_codes;

// ------ Frontend stuff ---------------------------------------------------------------------------

#[cfg(feature = "ink")]
mod frontend;

#[cfg(feature = "ink")]
pub use {
    frontend::{SenterpriseError, SenterpriseExtension, Environment},
    sp_core::H256 as KeyHash,
};

// ------ Backend stuff ----------------------------------------------------------------------------

#[cfg(feature = "runtime")]
mod backend;

#[cfg(feature = "runtime-benchmarks")]
pub use backend::ChainExtensionBenchmarking;
#[cfg(feature = "runtime")]
pub use {backend::SenterpriseChainExtension, module_vk_storage::KeyHash};
