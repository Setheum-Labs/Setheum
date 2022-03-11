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

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
use crate::{
	AccountId, Balance, CurrencyId, Currencies, dollar, Dex,
	MaxSwapSlippageCompareToOracle, Prices, Ratio, Runtime,
	SerpTreasury, StableCurrencyIds, StableCurrencyInflationPeriod,
	System, GetDinarCurrencyId, GetSerpCurrencyId, GetNativeCurrencyId,
	GetHelpCurrencyId, GetSetUSDId, SetterCurrencyId, 
};

use super::utils::set_balance;
use frame_benchmarking::whitelisted_caller;
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use frame_support::traits::OnInitialize;
use orml_traits::MultiCurrency;
use sp_runtime::traits::Zero;
use sp_std::prelude::*;
use module_support::{DEXManager, SerpTreasury as SerpTreasurySupport, SerpTreasuryExtended, SwapLimit};

const SETM: CurrencyId = GetNativeCurrencyId::get();
const SETR: CurrencyId = SetterCurrencyId::get();
const SETUSD: CurrencyId = GetSetUSDId::get();
const DNAR: CurrencyId = GetDinarCurrencyId::get();
const SERP: CurrencyId = GetSerpCurrencyId::get();
const HELP: CurrencyId = GetHelpCurrencyId::get();

runtime_benchmarks! {
	{ Runtime, serp_treasury }
	on_initialize {
		let c in 0 .. StableCurrencyIds::get().len().saturating_sub(1) as u32;
		let currency_ids = StableCurrencyIds::get();

		let block_number = StableCurrencyInflationPeriod::get();
		
		let caller: AccountId = whitelisted_caller();
		set_balance(DNAR, &caller, 1000000000 * dollar(DNAR));
		set_balance(SERP, &caller, 1000000000 * dollar(SERP));
		set_balance(SETM, &caller, 1000000000 * dollar(SETM));
		set_balance(HELP, &caller, 1000000000 * dollar(HELP));
		set_balance(SETR, &caller, 1000000000 * dollar(SETR));
		set_balance(SETUSD, &caller, 1000000000 * dollar(SETUSD));
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETR, DNAR);
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETR, SERP);
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETR, SETM);
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETR, HELP);
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETUSD, SETR);
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETUSD, DNAR);
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETUSD, SERP);
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETUSD, SETM);
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETUSD, HELP);
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETR,
			DNAR,
			1000 * dollar(SETR),
			100 * dollar(DNAR),
			0,
		)?;
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETR,
			SERP,
			1000 * dollar(SETR),
			100 * dollar(SERP),
			0,
		)?;
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETR,
			SETM,
			1000 * dollar(SETR),
			100 * dollar(SETM),
			0,
		)?;
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETR,
			HELP,
			1000 * dollar(SETR),
			100 * dollar(HELP),
			0,
		)?;
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETUSD,
			SETR,
			1000 * dollar(SETUSD),
			100 * dollar(SETR),
			0,
		)?;Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETUSD,
			DNAR,
			1000 * dollar(SETUSD),
			100 * dollar(DNAR),
			0,
		)?;
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETUSD,
			SERP,
			1000 * dollar(SETUSD),
			100 * dollar(SERP),
			0,
		)?;
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETUSD,
			SETM,
			1000 * dollar(SETUSD),
			100 * dollar(SETM),
			0,
		)?;
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETUSD,
			HELP,
			1000 * dollar(SETUSD),
			100 * dollar(HELP),
			0,
		)?;

		for i in 0 .. c {

			let currency_id = currency_ids[i as usize];
			
			let one: Balance = 1;
			let inflation_amount = SerpTreasury::stable_currency_inflation_rate(currency_id);
			let inflamounts: Balance = one.saturating_mul(inflation_amount / 5);

			if inflation_amount != 0 {
				
				// inflation distros
				// 1
				<SerpTreasury as SerpTreasurySupport<AccountId>>::add_cashdrop_to_pool(currency_id, inflamounts)?;
				// 2
				<SerpTreasury as SerpTreasuryExtended<AccountId>>::buyback_swap_with_exact_supply(
					currency_id,
					DNAR,
					SwapLimit::ExactSupply(inflamounts, 0),
				)?;
				// 3
				<SerpTreasury as SerpTreasuryExtended<AccountId>>::buyback_swap_with_exact_supply(
					currency_id,
					SERP,
					SwapLimit::ExactSupply(inflamounts, 0),
				)?;
				// 4
				<SerpTreasury as SerpTreasuryExtended<AccountId>>::buyback_swap_with_exact_supply(
					currency_id,
					SETM,
					SwapLimit::ExactSupply(inflamounts, 0),
				)?;
				// 5
				<SerpTreasury as SerpTreasuryExtended<AccountId>>::buyback_swap_with_exact_supply(
					currency_id,
					HELP,
					SwapLimit::ExactSupply(inflamounts, 0),
				)?;
			};
		}

		let (setter_pool, setdollar_pool) = Dex::get_liquidity_pool(SETR, SETUSD);

		let setter_peg: Balance = 4;

		let base_unit = setter_pool.saturating_mul(setter_peg);

		match setdollar_pool {
			0 => {} 
			setdollar_pool if setdollar_pool > base_unit => {
				// safe from underflow because `setdollar_pool` is checked to be greater than `base_unit`
				let supply = <Currencies as MultiCurrency<_>>::total_issuance(SETR);
				let expand_by = <SerpTreasury as SerpTreasurySupport<AccountId>>::calculate_supply_change(setdollar_pool, base_unit, supply);
				<SerpTreasury as SerpTreasurySupport<AccountId>>::on_serpup(SETR, expand_by)?;
			}
			setdollar_pool if setdollar_pool < base_unit => {
				// safe from underflow because `setdollar_pool` is checked to be less than `base_unit`
				let supply = <Currencies as MultiCurrency<_>>::total_issuance(SETR);
				let contract_by = <SerpTreasury as SerpTreasurySupport<AccountId>>::calculate_supply_change(base_unit, setdollar_pool, supply);
				<SerpTreasury as SerpTreasurySupport<AccountId>>::on_serpdown(SETR, contract_by)?;
			}
			_ => {}
		}

		SerpTreasury::on_initialize(1);
		System::set_block_number(block_number);
	}: {
		SerpTreasury::on_initialize(System::block_number());
	}

	set_stable_currency_inflation_rate {
	}: _(RawOrigin::Root, crate::SerpStableCurrencyId::SETR, 200 * 1_000_000_000_000_000_000)
	
	force_serpdown {
		let caller: AccountId = whitelisted_caller();
		set_balance(DNAR, &caller, 1000000000 * dollar(DNAR));
		set_balance(SERP, &caller, 1000000000 * dollar(SERP));
		set_balance(SETR, &caller, 1000000000 * dollar(SETR));
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETR, DNAR);
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), SETR, SERP);
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETR,
			DNAR,
			1000 * dollar(SETR),
			100 * dollar(DNAR),
			0,
		)?;
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			SETR,
			SERP,
			1000 * dollar(SETR),
			100 * dollar(SERP),
			0,
		)?;
	}: _(RawOrigin::Root, SETR, 100 * dollar(SETR))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
