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
	dollar, SetheumOracle, AccountId, Amount, SettmintEngine, StandardCurrencyIds, CurrencyId, SettmintGateway, Indices, Price, Rate,
	Ratio, Runtime, USDJ, SETR,
};

use super::utils::set_balance;
use core::convert::TryInto;
use frame_benchmarking::account;
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::Change;
use sp_runtime::{
	traits::{StaticLookup, UniqueSaturatedInto},
	FixedPointNumber,
};
use sp_std::prelude::*;

const SEED: u32 = 0;

runtime_benchmarks! {
	{ Runtime, settmint_gateway }

	_ {}

	authorize {
		let caller: AccountId = account("caller", 0, SEED);
		let to: AccountId = account("to", 0, SEED);
		let to_lookup = Indices::unlookup(to);
	}: _(RawOrigin::Signed(caller), SETR, to_lookup)

	unauthorize {
		let caller: AccountId = account("caller", 0, SEED);
		let to: AccountId = account("to", 0, SEED);
		let to_lookup = Indices::unlookup(to);
		SettmintGateway::authorize(
			RawOrigin::Signed(caller.clone()).into(),
			SETR,
			to_lookup.clone()
		)?;
	}: _(RawOrigin::Signed(caller), SETR, to_lookup)

	unauthorize_all {
		let c in 0 .. StandardCurrencyIds::get().len().saturating_sub(1) as u32;

		let caller: AccountId = account("caller", 0, SEED);
		let currency_ids = StandardCurrencyIds::get();
		let to: AccountId = account("to", 0, SEED);
		let to_lookup = Indices::unlookup(to);

		for i in 0 .. c {
			SettmintGateway::authorize(
				RawOrigin::Signed(caller.clone()).into(),
				currency_ids[i as usize],
				to_lookup.clone(),
			)?;
		}
	}: _(RawOrigin::Signed(caller))

	// `adjust_position`, best case:
	// adjust both reserve and standard
	adjust_position {
		let caller: AccountId = account("caller", 0, SEED);
		let currency_id: CurrencyId = StandardCurrencyIds::get()[0];
		let reserve_price = Price::one();		// 1 USD
		let standard_value = 100 * dollar(USDJ);
		let standard_exchange_rate = SettmintEngine::get_standard_exchange_rate(currency_id);
		let standard_amount = standard_exchange_rate.reciprocal().unwrap().saturating_mul_int(standard_value);
		let standard_amount: Amount = standard_amount.unique_saturated_into();
		let reserve_value = 10 * standard_value;
		let reserve_amount = Price::saturating_from_rational(dollar(currency_id), dollar(USDJ)).saturating_mul_int(reserve_value);

		// set balance
		set_balance(currency_id, &caller, reserve_amount);

		// feed price
		SetheumOracle::feed_values(RawOrigin::Root.into(), vec![(currency_id, reserve_price)])?;
	}: _(RawOrigin::Signed(caller), currency_id, reserve_amount.try_into().unwrap(), standard_amount)

	transfer_position_from {
		let currency_id: CurrencyId = StandardCurrencyIds::get()[0];
		let sender: AccountId = account("sender", 0, SEED);
		let sender_lookup = Indices::unlookup(sender.clone());
		let receiver: AccountId = account("receiver", 0, SEED);
		let receiver_lookup = Indices::unlookup(receiver.clone());


		let standard_value = 100 * dollar(USDJ);
		let standard_exchange_rate = SettmintEngine::get_standard_exchange_rate(currency_id);
		let standard_amount = standard_exchange_rate.reciprocal().unwrap().saturating_mul_int(standard_value);
		let standard_amount: Amount = standard_amount.unique_saturated_into();
		let reserve_value = 10 * standard_value;
		let reserve_amount = Price::saturating_from_rational(dollar(currency_id), dollar(USDJ)).saturating_mul_int(reserve_value);

		// set balance
		set_balance(currency_id, &sender, reserve_amount);

		// feed price
		SetheumOracle::feed_values(RawOrigin::Root.into(), vec![(currency_id, Price::one())])?;

		// initialize sender's setter
		SettmintGateway::adjust_position(
			RawOrigin::Signed(sender.clone()).into(),
			currency_id,
			reserve_amount.try_into().unwrap(),
			standard_amount,
		)?;

		// authorize receiver
		SettmintGateway::authorize(
			RawOrigin::Signed(sender.clone()).into(),
			currency_id,
			receiver_lookup,
		)?;
	}: _(RawOrigin::Signed(receiver), currency_id, sender_lookup)
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
	fn test_authorize() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_authorize());
		});
	}

	#[test]
	fn test_unauthorize() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_unauthorize());
		});
	}

	#[test]
	fn test_unauthorize_all() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_unauthorize_all());
		});
	}

	#[test]
	fn test_adjust_position() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_adjust_position());
		});
	}
}
