// This file is part of Setheum.

// Copyright (C) 2019-2021 Setheum Labs.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module exposes the Dinar halving issuance model in `P_NPoS` (Payout NPoS).
#![cfg_attr(not(feature = "std"), no_std)]

/// A trait for types that can provide the amount of issuance to award to the stakers.
pub trait Issuance<u64, Balance> {
	fn issuance(era_duration: u64) -> Balance;
}

// Minimal implementations for when you don't actually want any issuance
impl Issuance<u64, Balance> for () {
	fn issuance(_era_duration: u64) -> Balance {
		0
	}
}

impl Issuance<u64, Balance> for () {
	fn issuance(_era_duration: u64) -> Balance { 0 }
}

/// A type that provides era issuance according to setheum's rules
/// Initial issuance is 258 / era
/// Issuance is cut in half every 210,000 eras
pub struct SetheumHalving;

/// The number of eras between each halvening,
/// 4,032 eras (2 years) halving interval.
const HALVING_INTERVAL: u64 = 4032;
/// The per-block issuance before any halvenings. Decimal places should be accounted for here.
const INITIAL_ISSUANCE: u64 = 258;

impl Issuance<u64, Balance> for SetheumHalving {

	fn issuance(era_duration: u64) -> Balance {
		let halvings = era_duration / HALVING_INTERVAL;
		let halving_interval = T::
		let halvings = era_duration / halving_interval;
		// Force era reward to zero when right shift is undefined.
		if halvings >= 64 {
			return 0;
		}

		// Subsidy is cut in half every 4,032 eras which will occur
		// approximately every 2 years.
		(INITIAL_ISSUANCE >> halvings).into()
	}
}
