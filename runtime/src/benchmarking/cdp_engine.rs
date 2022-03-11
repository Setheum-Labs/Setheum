// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

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

use crate::{
	dollar, AccountId, Address, Amount, Balance, CdpEngine, CollateralCurrencyIds, CurrencyId,
	DefaultDebitExchangeRate, Dex, EmergencyShutdown, ExistentialDeposits, GetSetUSDId, GetDinarCurrencyId,
	MaxSwapSlippageCompareToOracle, MinimumDebitValue, Price, Rate, Ratio, Runtime,
};

use super::utils::{feed_price, set_balance};
use core::convert::TryInto;
use frame_benchmarking::account;
use frame_support::traits::OnInitialize;
use frame_system::RawOrigin;
use module_support::DEXManager;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::{Change, GetByKey};
use sp_runtime::{
	traits::{AccountIdLookup, One, StaticLookup, UniqueSaturatedInto},
	FixedPointNumber,
};
use sp_std::prelude::*;

const SEED: u32 = 0;

const STABLECOIN: CurrencyId = GetSetUSDId::get();
const DINARID: CurrencyId = GetDinarCurrencyId::get();

fn inject_liquidity(
	maker: AccountId,
	currency_id: CurrencyId,
	max_amount: Balance,
	max_other_currency_amount: Balance,
) -> Result<(), &'static str> {
	let base_currency_id = GetSetUSDId::get();

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
		Default::default(),
	)?;

	Ok(())
}

runtime_benchmarks! {
	{ Runtime, cdp_engine }

	set_collateral_params {
	}: _(
		RawOrigin::Root,
		DINARID,
		Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
		Change::NewValue(Some(Rate::saturating_from_rational(20, 100))),
		Change::NewValue(Some(Ratio::saturating_from_rational(180, 100))),
		Change::NewValue(100_000 * dollar(STABLECOIN))
	)

	// `liquidate` by_auction
	liquidate_by_auction {
		let owner: AccountId = account("owner", 0, SEED);
		let owner_lookup = AccountIdLookup::unlookup(owner.clone());
		let min_debit_value = MinimumDebitValue::get();
		let debit_exchange_rate = CdpEngine::get_debit_exchange_rate(DINARID);
		let collateral_price = Price::one();		// 1 USD
		let min_debit_amount = debit_exchange_rate.reciprocal().unwrap().saturating_mul_int(min_debit_value);
		let min_debit_amount: Amount = min_debit_amount.unique_saturated_into();
		let collateral_value = 2 * min_debit_value;
		let collateral_amount = Price::saturating_from_rational(dollar(DINARID), dollar(STABLECOIN)).saturating_mul_int(collateral_value);

		// set balance
		set_balance(DINARID, &owner, collateral_amount + ExistentialDeposits::get(&DINARID));

		// feed price
		feed_price(vec![(DINARID, collateral_price)])?;

		// set risk params
		CdpEngine::set_collateral_params(
			RawOrigin::Root.into(),
			DINARID,
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(Some(Rate::saturating_from_rational(10, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(min_debit_value * 100),
		)?;

		// adjust position
		CdpEngine::adjust_position(&owner, DINARID, collateral_amount.try_into().unwrap(), min_debit_amount)?;

		// modify liquidation rate to make the cdp unsafe
		CdpEngine::set_collateral_params(
			RawOrigin::Root.into(),
			DINARID,
			Change::NewValue(Some(Ratio::saturating_from_rational(1000, 100))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		)?;
	}: liquidate(RawOrigin::None, DINARID, owner_lookup)

	// `liquidate` by dex
	liquidate_by_dex {
		let owner: AccountId = account("owner", 0, SEED);
		let owner_lookup = AccountIdLookup::unlookup(owner.clone());
		let funder: AccountId = account("funder", 0, SEED);

		let debit_value = 100 * dollar(STABLECOIN);
		let debit_exchange_rate = CdpEngine::get_debit_exchange_rate(DINARID);
		let debit_amount = debit_exchange_rate.reciprocal().unwrap().saturating_mul_int(debit_value);
		let debit_amount: Amount = debit_amount.unique_saturated_into();
		let collateral_value = 2 * debit_value;
		let collateral_amount = Price::saturating_from_rational(dollar(DINARID), dollar(STABLECOIN)).saturating_mul_int(collateral_value);
		let collateral_price = Price::one();		// 1 USD
		let max_slippage_swap_with_dex = MaxSwapSlippageCompareToOracle::get();
		let collateral_amount_in_dex = max_slippage_swap_with_dex.reciprocal().unwrap().saturating_mul_int(collateral_amount);
		let base_amount_in_dex = max_slippage_swap_with_dex.reciprocal().unwrap().saturating_mul_int(debit_value * 2);

		inject_liquidity(funder.clone(), DINARID, base_amount_in_dex, collateral_amount_in_dex)?;

		// set balance
		set_balance(DINARID, &owner, collateral_amount + ExistentialDeposits::get(&DINARID));

		// feed price
		feed_price(vec![(DINARID, collateral_price)])?;

		// set risk params
		CdpEngine::set_collateral_params(
			RawOrigin::Root.into(),
			DINARID,
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(Some(Rate::saturating_from_rational(10, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(debit_value * 100),
		)?;

		// adjust position
		CdpEngine::adjust_position(&owner, DINARID, collateral_amount.try_into().unwrap(), debit_amount)?;

		// modify liquidation rate to make the cdp unsafe
		CdpEngine::set_collateral_params(
			RawOrigin::Root.into(),
			DINARID,
			Change::NewValue(Some(Ratio::saturating_from_rational(1000, 100))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		)?;
	}: liquidate(RawOrigin::None, DINARID, owner_lookup)
	verify {
		let (other_currency_amount, base_currency_amount) = Dex::get_liquidity_pool(DINARID, STABLECOIN);
		assert!(other_currency_amount > collateral_amount_in_dex);
		assert!(base_currency_amount < base_amount_in_dex);
	}

	settle {
		let owner: AccountId = account("owner", 0, SEED);
		let owner_lookup = AccountIdLookup::unlookup(owner.clone());
		let min_debit_value = MinimumDebitValue::get();
		let debit_exchange_rate = CdpEngine::get_debit_exchange_rate(DINARID);
		let collateral_price = Price::one();		// 1 USD
		let min_debit_amount = debit_exchange_rate.reciprocal().unwrap().saturating_mul_int(min_debit_value);
		let min_debit_amount: Amount = min_debit_amount.unique_saturated_into();
		let collateral_value = 2 * min_debit_value;
		let collateral_amount = Price::saturating_from_rational(dollar(DINARID), dollar(STABLECOIN)).saturating_mul_int(collateral_value);

		// set balance
		set_balance(DINARID, &owner, collateral_amount + ExistentialDeposits::get(&DINARID));

		// feed price
		feed_price(vec![(DINARID, collateral_price)])?;

		// set risk params
		CdpEngine::set_collateral_params(
			RawOrigin::Root.into(),
			DINARID,
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(Some(Rate::saturating_from_rational(10, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(150, 100))),
			Change::NewValue(min_debit_value * 100),
		)?;

		// adjust position
		CdpEngine::adjust_position(&owner, DINARID, collateral_amount.try_into().unwrap(), min_debit_amount)?;

		// shutdown
		EmergencyShutdown::emergency_shutdown(RawOrigin::Root.into())?;
	}: _(RawOrigin::None, DINARID, owner_lookup)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
