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

use crate::{dollar, CashDrop, Currencies, CurrencyId, Runtime, DNAR, USDJ, SETT};

use frame_system::RawOrigin;
use setheum_support::CashDrop;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::MultiCurrency;
use sp_std::prelude::*;

runtime_benchmarks! {
	{ Runtime, serp_treasury }

	_ {}

	update_minimum_claimable_transfer {
		let c in 0 .. RewardableCurrencyIds::get().len().saturating_sub(1) as u32;
		let currency_ids = RewardableCurrencyIds::get();
		let caller: AccountId = account("caller", 0, SEED);
		let mut values = vec![];
		let base_currency_id = GetStableCurrencyId::get();

		for i in 0 .. c {
			let currency_id = currency_ids[i as usize];
			values.push((currency_id, 100 * dollar(DNAR)));
		}
	}: _(RawOrigin::Root, values)
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
	fn update_minimum_claimable_transfer() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_update_minimum_claimable_transfer());
		});
	}
}
