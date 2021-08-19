// This file is part of Setheum.

// Copyright (C) 2019-2021 Setheum Labs.
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

//! Common runtime code for Setheum and NewRome.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	parameter_types,
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_MILLIS},
		DispatchClass, Weight,
	},
};
use frame_system::limits;
pub use setheum_support::{ExchangeRate, PrecompileCallerFilter, Price, Rate, Ratio};
use primitives::{
	Balance, CurrencyId, PRECOMPILE_ADDRESS_START, PREDEPLOY_ADDRESS_START, SYSTEM_CONTRACT_ADDRESS_PREFIX,
};
use sp_core::H160;
use sp_runtime::{
	// TODO: move after https://github.com/paritytech/substrate/pull/9209
	offchain::storage_lock::BlockNumberProvider,
	traits::Convert,
	transaction_validity::TransactionPriority,
	Perbill,
};
use static_assertions::const_assert;

pub mod precompile;
pub use precompile::{
	AllPrecompiles, DexPrecompile, MultiCurrencyPrecompile, NFTPrecompile, OraclePrecompile, ScheduleCallPrecompile,
	StateRentPrecompile,
};
pub use primitives::currency::{
	TokenInfo, 
	DNAR, DRAM, SETR, SETUSD, SETEUR, SETGBP, SETCHF, SEKJ, SGDJ, SETSAR, RENBTC,
	USD, EUR, GBP, CHF, SAR, KWD, JOD, BHD, KYD, OMR, GIP,
};

pub type TimeStampedPrice = orml_oracle::TimestampedValue<Price, primitives::Moment>;

// Priority of unsigned transactions
parameter_types! {
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	pub const RenvmBridgeUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	pub const SettmintEngineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
}

/// Check if the given `address` is a system contract.
///
/// It's system contract if the address starts with SYSTEM_CONTRACT_ADDRESS_PREFIX.
pub fn is_system_contract(address: H160) -> bool {
	address.as_bytes().starts_with(&SYSTEM_CONTRACT_ADDRESS_PREFIX)
}

pub fn is_setheum_precompile(address: H160) -> bool {
	address >= H160::from_low_u64_be(PRECOMPILE_ADDRESS_START)
		&& address < H160::from_low_u64_be(PREDEPLOY_ADDRESS_START)
}

/// The call is allowed only if caller is a system contract.
pub struct SystemContractsFilter;
impl PrecompileCallerFilter for SystemContractsFilter {
	fn is_allowed(caller: H160) -> bool {
		is_system_contract(caller)
	}
}

/// Convert gas to weight
pub struct GasToWeight;
impl Convert<u64, Weight> for GasToWeight {
	fn convert(a: u64) -> u64 {
		// TODO: estimate this
		a as Weight
	}
}

// TODO: somehow estimate this value. Start from a conservative value.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_perthousand(25);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be
/// used by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 1 second of compute with a 3 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 1 * WEIGHT_PER_SECOND;

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

parameter_types! {
	/// Maximum length of block. Up to 5MB.
	pub BlockLength: limits::BlockLength =
		limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	/// Block weights base values and limits.
	pub BlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have an extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}

parameter_types! {
	/// A limit for off-chain phragmen unsigned solution submission.
	///
	/// We want to keep it as high as possible, but can't risk having it reject,
	/// so we always subtract the base block execution weight.
	pub OffchainSolutionWeightLimit: Weight = BlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic
		.expect("Normal extrinsics have weight limit configured by default; qed")
		.saturating_sub(BlockExecutionWeight::get());
}

// TODO: make those const fn
pub fn dollar(currency_id: CurrencyId) -> Balance {
	10u128.saturating_pow(currency_id.decimals().expect("Not support Erc20 decimals").into())
}

pub fn cent(currency_id: CurrencyId) -> Balance {
	dollar(currency_id) / 100
}

pub fn millicent(currency_id: CurrencyId) -> Balance {
	cent(currency_id) / 1000
}

pub fn microcent(currency_id: CurrencyId) -> Balance {
	millicent(currency_id) / 1000
}

pub fn deposit(items: u32, bytes: u32, currency_id: CurrencyId) -> Balance {
	items as Balance * 15 * cent(currency_id) + (bytes as Balance) * 6 * cent(currency_id)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn system_contracts_filter_works() {
		assert!(SystemContractsFilter::is_allowed(H160::from_low_u64_be(1)));

		let mut max_allowed_addr = [0u8; 20];
		max_allowed_addr[SYSTEM_CONTRACT_ADDRESS_PREFIX.len()] = 127u8;
		assert!(SystemContractsFilter::is_allowed(max_allowed_addr.into()));

		let mut min_blocked_addr = [0u8; 20];
		min_blocked_addr[SYSTEM_CONTRACT_ADDRESS_PREFIX.len() - 1] = 1u8;
		assert!(!SystemContractsFilter::is_allowed(min_blocked_addr.into()));
	}

	#[test]
	fn is_system_contract_works() {
		assert!(is_system_contract(H160::from_low_u64_be(0)));
		assert!(is_system_contract(H160::from_low_u64_be(u64::max_value())));

		let mut bytes = [0u8; 20];
		bytes[SYSTEM_CONTRACT_ADDRESS_PREFIX.len() - 1] = 1u8;

		assert!(!is_system_contract(bytes.into()));

		bytes = [0u8; 20];
		bytes[0] = 1u8;

		assert!(!is_system_contract(bytes.into()));
	}

	#[test]
	fn is_setheum_precompile_works() {
		assert!(!is_setheum_precompile(H160::from_low_u64_be(0)));
		assert!(!is_setheum_precompile(H160::from_low_u64_be(
			PRECOMPILE_ADDRESS_START - 1
		)));
		assert!(is_setheum_precompile(H160::from_low_u64_be(PRECOMPILE_ADDRESS_START)));
		assert!(is_setheum_precompile(H160::from_low_u64_be(PREDEPLOY_ADDRESS_START - 1)));
		assert!(!is_setheum_precompile(H160::from_low_u64_be(PREDEPLOY_ADDRESS_START)));
		assert!(!is_setheum_precompile([1u8; 20].into()));
	}
}
