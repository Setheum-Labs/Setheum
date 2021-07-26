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

use crate::{dollar, SerpTreasury, Currencies, CurrencyId, Runtime, DNAR, USDJ, SETT};

use frame_system::RawOrigin;
use setheum_support::SerpTreasury;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::MultiCurrency;
use sp_std::prelude::*;

runtime_benchmarks! {
	{ Runtime, serp_treasury }

	_ {}

	auction_serplus {// TODO: update
		Currencies::deposit(SETT, &SerpTreasury::account_id(), 10_000 * dollar(SETT))?;
	}: _(RawOrigin::Root, SETT, 1_000 * dollar(SETT), 1_000 * dollar(USDJ), true)

	auction_diamond { // TODO: update
		let currency_id: CurrencyId = USDJ;
		Currencies::deposit(SETT, &SerpTreasury::account_id(), 10_000 * dollar(SETT))?;
	}: _(RawOrigin::Root, SETT, 1_000 * dollar(SETT), 1_000 * dollar(USDJ), true)

	auction_setter {
		Currencies::deposit(SETT, &SerpTreasury::account_id(), 10_000 * dollar(SETT))?;
	}: _(RawOrigin::Root, SETT, 1_000 * dollar(SETT), 1_000 * dollar(USDJ), true)
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
	fn test_auction_serplus() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_auction_serplus());
		});
	}

	#[test]
	fn test_auction_diamond() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_auction_diamond());
		});
	}

	#[test]
	fn test_auction_setter() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_auction_setter());
		});
	}
}
