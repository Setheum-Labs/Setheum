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

//! Unit tests for the prices module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};use sp_runtime::{traits::{BadOrigin, Zero}, FixedPointNumber};

#[test]
fn get_price_from_oracle() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			PricesModule::get_price(JCHF),
			Some(Price::saturating_from_integer(500000000000000u128))
		); // 50000 USD, right shift the decimal point (18-10) places
		assert_eq!(
			PricesModule::get_price(DNAR),
			Some(Price::saturating_from_integer(10000000000u128))
		); // 100 USD, right shift the decimal point (18-12) places
		assert_eq!(PricesModule::get_price(DNAR), Some(Price::zero()));
	});
}

#[test]
fn get_price_of_stable_currency_id() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			PricesModule::get_price(USDJ),
			Some(Price::saturating_from_integer(1000000))
		); // 1 USD, right shift the decimal point (18-12) places
	});
}

#[test]
fn get_price_of_liquid_currency_id() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			PricesModule::get_price(LDNAR),
			Some(Price::saturating_from_integer(5000000000u128))
		); // 50 USD, right shift the decimal point (18-12) places
	});
}

#[test]
fn get_price_of_lp_token_currency_id() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(MockSetheumDex::get_liquidity_pool(USDJ, DNAR), (10000, 200));
		assert_eq!(
			PricesModule::get_price(LP_USDJ_DNAR),
			None
		);
		assert_ok!(Tokens::deposit(LP_USDJ_DNAR, &1, 100));
		assert_eq!(Tokens::total_issuance(LP_USDJ_DNAR), 100);
		assert_eq!(PricesModule::get_price(USDJ), Some(Price::saturating_from_rational(1000000u128, 1)));
		assert_eq!(
			PricesModule::get_price(LP_USDJ_DNAR),
			Some(Price::saturating_from_rational(200000000u128, 1))	// 10000/100 * Price::saturating_from_rational(1000000u128, 1) * 2
		);

		assert_eq!(MockSetheumDex::get_liquidity_pool(JCHF, USDJ), (0, 0));
		assert_eq!(
			PricesModule::get_price(LP_JCHF_USDJ),
			None
		);
	});
}

#[test]
fn get_relative_price_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			PricesModule::get_relative_price(DNAR, USDJ),
			Some(Price::saturating_from_rational(10000, 1)) /* 1DNAR = 100USDJ, right shift the decimal point (12-10)
			                                                 * places */
		);
		assert_eq!(
			PricesModule::get_relative_price(JCHF, USDJ),
			Some(Price::saturating_from_rational(500000000, 1)) /* 1JCHF = 50000USDJ, right shift the decimal point
			                                                     * (12-8) places */
		);
		assert_eq!(
			PricesModule::get_relative_price(LDNAR, DNAR),
			Some(Price::saturating_from_rational(1, 2)) // 1LDNAR = 1/2DNAR, right shift the decimal point (10-10) places
		);
		assert_eq!(
			PricesModule::get_relative_price(USDJ, USDJ),
			Some(Price::saturating_from_rational(1, 1)) // 1USDJ = 1USDJ, right shift the decimal point (10-10) places
		);
		assert_eq!(PricesModule::get_relative_price(USDJ, DNAR), None);
	});
}

#[test]
fn lock_price_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			PricesModule::get_price(JCHF),
			Some(Price::saturating_from_integer(500000000000000u128))
		);
		LockedPrice::<Runtime>::insert(JCHF, Price::saturating_from_integer(80000));
		assert_eq!(
			PricesModule::get_price(JCHF),
			Some(Price::saturating_from_integer(800000000000000u128))
		);
	});
}

#[test]
fn lock_price_call_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(PricesModule::lock_price(Origin::signed(5), JCHF), BadOrigin,);
		assert_ok!(PricesModule::lock_price(Origin::signed(1), JCHF));

		let lock_price_event = Event::prices(crate::Event::LockPrice(JCHF, Price::saturating_from_integer(50000)));
		assert!(System::events().iter().any(|record| record.event == lock_price_event));
		assert_eq!(
			PricesModule::locked_price(JCHF),
			Some(Price::saturating_from_integer(50000))
		);
	});
}

#[test]
fn unlock_price_call_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		LockedPrice::<Runtime>::insert(JCHF, Price::saturating_from_integer(80000));
		assert_noop!(PricesModule::unlock_price(Origin::signed(5), JCHF), BadOrigin,);
		assert_ok!(PricesModule::unlock_price(Origin::signed(1), JCHF));

		let unlock_price_event = Event::prices(crate::Event::UnlockPrice(JCHF));
		assert!(System::events().iter().any(|record| record.event == unlock_price_event));

		assert_eq!(PricesModule::locked_price(JCHF), None);
	});
}
