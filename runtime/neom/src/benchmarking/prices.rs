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

use crate::{SetheumOracle, CollateralCurrencyId, CurrencyId, Origin, Price, Prices, Runtime, NEOM};

use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use sp_runtime::traits::One;
use sp_std::prelude::*;

runtime_benchmarks! {
	{ Runtime, module_prices }

	lock_price {
		let currency_id: CurrencyId = CollateralCurrencyId::get()[0];

		// feed price
		SetheumOracle::feed_values(RawOrigin::Root.into(), vec![(currency_id, Price::one())])?;
	}: _(RawOrigin::Root, NEOM)

	unlock_price {
		let currency_id: CurrencyId = CollateralCurrencyId::get()[0];

		// feed price
		SetheumOracle::feed_values(RawOrigin::Root.into(), vec![(currency_id, Price::one())])?;
		Prices::lock_price(Origin::root(), NEOM)?;
	}: _(RawOrigin::Root, NEOM)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
