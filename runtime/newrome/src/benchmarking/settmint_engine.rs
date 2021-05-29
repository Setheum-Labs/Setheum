// This file is part of Setheum.

// Copyright (C) 2020-2021 Setheum Labs.
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
	dollar, SetheumOracle, AccountId, Amount, Balance, SettmintEngine, CurrencyId, Dex, EmergencyShutdown,
	GetStableCurrencyId, Indices, MaxSlippageSwapWithDEX, MinimumStandardValue, Price, Rate, Ratio, Runtime, USDJ, DOT,
};

use super::utils::set_balance;
use core::convert::TryInto;
use frame_benchmarking::account;
use frame_system::RawOrigin;
use setheum_support::DEXManager;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::Change;
use sp_runtime::{
	traits::{StaticLookup, UniqueSaturatedInto},
	FixedPointNumber,
};
use sp_std::prelude::*;

const SEED: u32 = 0;

fn inject_liquidity(
	maker: AccountId,
	currency_id: CurrencyId,
	max_amount: Balance,
	max_other_currency_amount: Balance,
) -> Result<(), &'static str> {
	let base_currency_id = GetStableCurrencyId::get();

	// set balance
	set_balance(currency_id, &maker, max_other_currency_amount.unique_saturated_into());
	set_balance(base_currency_id, &maker, max_amount.unique_saturated_into());

	let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), currency_id, base_currency_id);

	Dex::add_liquidity(
		RawOrigin::Signed(maker.clone()).into(),
		base_currency_id,
		currency_id,
		max_amount,
		max_other_currency_amount,
		false,
	)?;

	Ok(())
}

runtime_benchmarks! {
	{ Runtime, setheum_settmint_engine }

	_ {}

	set_reserve_params {
	}: _(
		RawOrigin::Root,
		DOT,
		Change::NewValue(Some(Rate::saturating_from_rational(1, 1000000))),
		Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
		Change::NewValue(Some(Rate::saturating_from_rational(20, 100))),
		Change::NewValue(Some(Ratio::saturating_from_rational(180, 100))),
		Change::NewValue(100_000 * dollar(USDJ))
	)

	set_global_params {
	}: _(RawOrigin::Root, Rate::saturating_from_rational(1, 1000000))

	// `liquidate` by_auction
	liquidate_by_auction {
		let owner: AccountId = account("owner", 0, SEED);
		let owner_lookup = Indices::unlookup(owner.clone());
		let currency_id: CurrencyId = DOT;
		let min_standard_value = MinimumStandardValue::get();
		let standard_exchange_rate = SettmintEngine::get_standard_exchange_rate(currency_id);
		let reserve_price = Price::one();		// 1 USD
		let min_standard_amount = standard_exchange_rate.reciprocal().unwrap().saturating_mul_int(min_standard_value);
		let min_standard_amount: Amount = min_standard_amount.unique_saturated_into();
		let reserve_value = 2 * min_standard_value;
		let reserve_amount = Price::saturating_from_rational(dollar(DOT), dollar(USDJ)).saturating_mul_int(reserve_value);

		// set balance
		set_balance(currency_id, &owner, reserve_amount);

		// feed price
		SetheumOracle::feed_values(RawOrigin::Root.into(), vec![(currency_id, reserve_price)])?;

		// set risk params
		SettmintEngine::set_reserve_params(
			RawOrigin::Root.into(),
			currency_id,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(Some(Rate::saturating_from_rational(10, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(min_standard_value * 100),
		)?;

		// adjust position
		SettmintEngine::adjust_position(&owner, currency_id, reserve_amount.try_into().unwrap(), min_standard_amount)?;

		// modify liquidation rate to make the settmint unsafe
		SettmintEngine::set_reserve_params(
			RawOrigin::Root.into(),
			currency_id,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(1000, 100))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		)?;
	}: liquidate(RawOrigin::None, currency_id, owner_lookup)

	// `liquidate` by dex
	liquidate_by_dex {
		let owner: AccountId = account("owner", 0, SEED);
		let owner_lookup = Indices::unlookup(owner.clone());
		let funder: AccountId = account("funder", 0, SEED);

		let standard_value = 100 * dollar(USDJ);
		let standard_exchange_rate = SettmintEngine::get_standard_exchange_rate(DOT);
		let standard_amount = standard_exchange_rate.reciprocal().unwrap().saturating_mul_int(standard_value);
		let standard_amount: Amount = standard_amount.unique_saturated_into();
		let reserve_value = 2 * standard_value;
		let reserve_amount = Price::saturating_from_rational(dollar(DOT), dollar(USDJ)).saturating_mul_int(reserve_value);
		let reserve_price = Price::one();		// 1 USD
		let max_slippage_swap_with_dex = MaxSlippageSwapWithDEX::get();
		let reserve_amount_in_dex = max_slippage_swap_with_dex.reciprocal().unwrap().saturating_mul_int(reserve_amount);
		let base_amount_in_dex = max_slippage_swap_with_dex.reciprocal().unwrap().saturating_mul_int(standard_value * 2);

		inject_liquidity(funder.clone(), DOT, base_amount_in_dex, reserve_amount_in_dex)?;

		// set balance
		set_balance(DOT, &owner, reserve_amount);

		// feed price
		SetheumOracle::feed_values(RawOrigin::Root.into(), vec![(DOT, reserve_price)])?;

		// set risk params
		SettmintEngine::set_reserve_params(
			RawOrigin::Root.into(),
			DOT,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(Some(Rate::saturating_from_rational(10, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(standard_value * 100),
		)?;

		// adjust position
		SettmintEngine::adjust_position(&owner, DOT, reserve_amount.try_into().unwrap(), standard_amount)?;

		// modify liquidation rate to make the settmint unsafe
		SettmintEngine::set_reserve_params(
			RawOrigin::Root.into(),
			DOT,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(1000, 100))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		)?;
	}: liquidate(RawOrigin::None, DOT, owner_lookup)
	verify {
		let (other_currency_amount, base_currency_amount) = Dex::get_liquidity_pool(DOT, USDJ);
		assert!(other_currency_amount > reserve_amount_in_dex);
		assert!(base_currency_amount < base_amount_in_dex);
	}

	settle {
		let owner: AccountId = account("owner", 0, SEED);
		let owner_lookup = Indices::unlookup(owner.clone());
		let currency_id: CurrencyId = DOT;
		let min_standard_value = MinimumStandardValue::get();
		let standard_exchange_rate = SettmintEngine::get_standard_exchange_rate(currency_id);
		let reserve_price = Price::one();		// 1 USD
		let min_standard_amount = standard_exchange_rate.reciprocal().unwrap().saturating_mul_int(min_standard_value);
		let min_standard_amount: Amount = min_standard_amount.unique_saturated_into();
		let reserve_value = 2 * min_standard_value;
		let reserve_amount = Price::saturating_from_rational(dollar(DOT), dollar(USDJ)).saturating_mul_int(reserve_value);

		// set balance
		set_balance(currency_id, &owner, reserve_amount);

		// feed price
		SetheumOracle::feed_values(RawOrigin::Root.into(), vec![(currency_id, reserve_price)])?;

		// set risk params
		SettmintEngine::set_reserve_params(
			RawOrigin::Root.into(),
			currency_id,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(Some(Rate::saturating_from_rational(10, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(min_standard_value * 100),
		)?;

		// adjust position
		SettmintEngine::adjust_position(&owner, currency_id, reserve_amount.try_into().unwrap(), min_standard_amount)?;

		// shutdown
		EmergencyShutdown::emergency_shutdown(RawOrigin::Root.into())?;
	}: _(RawOrigin::None, currency_id, owner_lookup)
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::assert_ok;

	fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap()
			.into()
	}

	#[test]
	fn test_set_reserve_params() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_reserve_params());
		});
	}

	#[test]
	fn test_set_global_params() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_global_params());
		});
	}

	#[test]
	fn test_liquidate_by_auction() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_liquidate_by_auction());
		});
	}

	#[test]
	fn test_liquidate_by_dex() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_liquidate_by_dex());
		});
	}

	#[test]
	fn test_settle() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_settle());
		});
	}
}
