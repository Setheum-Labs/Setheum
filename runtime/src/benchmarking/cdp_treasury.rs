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

use crate::{dollar, AccountId, CdpTreasury, Currencies, CurrencyId, Dex, GetNativeCurrencyId, GetSetUSDId, MaxAuctionsCount, Runtime};

use super::utils::set_balance;
use frame_benchmarking::whitelisted_caller;
use frame_system::RawOrigin;
use module_support::{CDPTreasury, SwapLimit};
use orml_benchmarking::runtime_benchmarks;
use orml_traits::MultiCurrency;

const STABLECOIN: CurrencyId = GetSetUSDId::get();
const SETMID: CurrencyId = GetNativeCurrencyId::get();

runtime_benchmarks! {
	{ Runtime, cdp_treasury }

	auction_collateral {
		let b in 1 .. MaxAuctionsCount::get();

		let auction_size = (1_000 * dollar(SETMID)) / b as u128;
		CdpTreasury::set_expected_collateral_auction_size(RawOrigin::Root.into(), SETMID, auction_size)?;

		Currencies::deposit(SETMID, &CdpTreasury::account_id(), 10_000 * dollar(SETMID))?;
	}: _(RawOrigin::Root, SETMID, 1_000 * dollar(SETMID), 1_000 * dollar(STABLECOIN), true)

	exchange_collateral_to_stable {
		let caller: AccountId = whitelisted_caller();
		set_balance(STABLECOIN, &caller, 1000 * dollar(STABLECOIN));
		set_balance(SETMID, &caller, 1000 * dollar(SETMID));
		let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), STABLECOIN, SETMID);
		Dex::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			STABLECOIN,
			SETMID,
			1000 * dollar(STABLECOIN),
			100 * dollar(SETMID),
			0,
		)?;
		CdpTreasury::deposit_collateral(&caller, SETMID, 100 * dollar(SETMID))?;
	}: _(RawOrigin::Root, SETMID, SwapLimit::ExactSupply(100 * dollar(SETMID), 0))

	set_expected_collateral_auction_size {
	}: _(RawOrigin::Root, SETMID, 200 * dollar(SETMID))

	extract_surplus_to_serp {
		CdpTreasury::on_system_surplus(1_000 * dollar(STABLECOIN))?;
	}: _(RawOrigin::Root, 200 * dollar(STABLECOIN))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
