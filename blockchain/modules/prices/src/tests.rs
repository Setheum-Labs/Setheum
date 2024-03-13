// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of Setheum.

// Copyright (C) 2019-Present Setheum Labs.
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
use mock::{RuntimeEvent, *};
use sp_runtime::{
	traits::{BadOrigin, Bounded},
	FixedPointNumber,
};

#[test]
fn lp_token_fair_price_works() {
	let lp_token_fair_price_0 = lp_token_fair_price(
		10000,
		20000,
		10000,
		Price::saturating_from_integer(100),
		Price::saturating_from_integer(200),
	)
	.unwrap();
	assert!(
		lp_token_fair_price_0 <= Price::saturating_from_integer(400)
			&& lp_token_fair_price_0 >= Price::saturating_from_integer(399)
	);

	assert_eq!(
		lp_token_fair_price(
			0,
			20000,
			10000,
			Price::saturating_from_integer(100),
			Price::saturating_from_integer(200)
		),
		None
	);
	assert_eq!(
		lp_token_fair_price(
			10000,
			0,
			10000,
			Price::saturating_from_integer(100),
			Price::saturating_from_integer(200)
		),
		Some(Price::from_inner(0))
	);
	assert_eq!(
		lp_token_fair_price(
			10000,
			20000,
			0,
			Price::saturating_from_integer(100),
			Price::saturating_from_integer(200)
		),
		Some(Price::from_inner(0))
	);
	assert_eq!(
		lp_token_fair_price(
			10000,
			20000,
			10000,
			Price::saturating_from_integer(100),
			Price::from_inner(0)
		),
		Some(Price::from_inner(0))
	);
	assert_eq!(
		lp_token_fair_price(
			10000,
			20000,
			10000,
			Price::from_inner(0),
			Price::saturating_from_integer(200)
		),
		Some(Price::from_inner(0))
	);

	assert_eq!(
		lp_token_fair_price(
			Balance::max_value(),
			Balance::max_value(),
			Balance::max_value(),
			Price::max_value() / Price::saturating_from_integer(2),
			Price::max_value() / Price::saturating_from_integer(2)
		),
		Some(Price::max_value() - Price::from_inner(1))
	);
	assert_eq!(
		lp_token_fair_price(
			Balance::max_value(),
			Balance::max_value(),
			Balance::max_value(),
			Price::max_value(),
			Price::max_value()
		),
		None
	);
}

#[test]
fn access_price_of_ussd() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			PricesModule::access_price(USSD),
			Some(Price::saturating_from_integer(1000000u128))
		); // 1 USD, right shift the decimal point (18-12) places

		mock_oracle_update();
		assert_eq!(
			PricesModule::access_price(USSD),
			Some(Price::saturating_from_integer(1000000u128))
		);
	});
}

#[test]
fn access_price_of_liquid_currency() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			PricesModule::access_price(SEE),
			Some(Price::saturating_from_integer(10000000000u128))
		); // 100 USD, right shift the decimal point (18-12) places
		assert_eq!(
			PricesModule::access_price(LSEE),
			Some(Price::saturating_from_integer(5000000000u128))
		); // see_price * 1/2

		mock_oracle_update();
		assert_eq!(
			PricesModule::access_price(SEE),
			Some(Price::saturating_from_integer(1000000000u128))
		); // 10 USD, right shift the decimal point (18-12) places
		assert_eq!(
			PricesModule::access_price(LSEE),
			Some(Price::saturating_from_integer(600000000u128))
		); // see_price * 3/5
	});
}

#[test]
fn access_price_of_dex_share_currency() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			PricesModule::access_price(SEE),
			Some(Price::saturating_from_integer(10000000000u128))
		); // 100 USD, right shift the decimal point (18-12) places
		assert_eq!(
			PricesModule::access_price(USSD),
			Some(Price::saturating_from_integer(1000000u128))
		);
		assert_eq!(Tokens::total_issuance(LP_USSD_SEE), 0);
		assert_eq!(MockDEX::get_liquidity_pool(USSD, SEE), (10000, 200));

		// when the total issuance of dex share currency is zero
		assert_eq!(PricesModule::access_price(LP_USSD_SEE), None);

		// issue LP
		assert_ok!(Tokens::deposit(LP_USSD_SEE, &1, 100));
		assert_eq!(Tokens::total_issuance(LP_USSD_SEE), 100);

		let lp_price_1 = lp_token_fair_price(
			Tokens::total_issuance(LP_USSD_SEE),
			MockDEX::get_liquidity_pool(USSD, SEE).0,
			MockDEX::get_liquidity_pool(USSD, SEE).1,
			PricesModule::access_price(USSD).unwrap(),
			PricesModule::access_price(SEE).unwrap(),
		);
		assert_eq!(PricesModule::access_price(LP_USSD_SEE), lp_price_1);

		// issue more LP
		assert_ok!(Tokens::deposit(LP_USSD_SEE, &1, 100));
		assert_eq!(Tokens::total_issuance(LP_USSD_SEE), 200);

		let lp_price_2 = lp_token_fair_price(
			Tokens::total_issuance(LP_USSD_SEE),
			MockDEX::get_liquidity_pool(USSD, SEE).0,
			MockDEX::get_liquidity_pool(USSD, SEE).1,
			PricesModule::access_price(USSD).unwrap(),
			PricesModule::access_price(SEE).unwrap(),
		);
		assert_eq!(PricesModule::access_price(LP_USSD_SEE), lp_price_2);

		mock_oracle_update();

		let lp_price_3 = lp_token_fair_price(
			Tokens::total_issuance(LP_USSD_SEE),
			MockDEX::get_liquidity_pool(USSD, SEE).0,
			MockDEX::get_liquidity_pool(USSD, SEE).1,
			PricesModule::access_price(USSD).unwrap(),
			PricesModule::access_price(SEE).unwrap(),
		);
		assert_eq!(PricesModule::access_price(LP_USSD_SEE), lp_price_3);
	});
}

#[test]
fn access_price_of_other_currency() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(PricesModule::access_price(SEE), Some(Price::saturating_from_integer(0)));
		assert_eq!(PricesModule::access_price(EDF), None);

		mock_oracle_update();

		assert_eq!(
			PricesModule::access_price(SEE),
			Some(Price::saturating_from_integer(30000000u128))
		); // 30 USD, right shift the decimal point (18-12) places
		assert_eq!(
			PricesModule::access_price(EDF),
			Some(Price::saturating_from_integer(200000000u128))
		); // 200 USD, right shift the decimal point (18-12) places
	});
}

#[test]
fn lock_price_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		// cannot lock the price of EDF when the price from oracle is None
		assert_eq!(PricesModule::access_price(EDF), None);
		assert_eq!(PricesModule::locked_price(EDF), None);
		assert_noop!(
			PricesModule::lock_price(RuntimeOrigin::signed(1), EDF),
			Error::<Runtime>::AccessPriceFailed
		);
		assert_eq!(PricesModule::locked_price(EDF), None);

		mock_oracle_update();

		// lock the price of EDF when the price of EDF from oracle is some
		assert_eq!(
			PricesModule::access_price(EDF),
			Some(Price::saturating_from_integer(200000000u128))
		);
		assert_eq!(PricesModule::locked_price(EDF), None);
		assert_ok!(PricesModule::lock_price(RuntimeOrigin::signed(1), EDF));
		System::assert_last_event(RuntimeEvent::PricesModule(crate::Event::LockPrice {
			currency_id: EDF,
			locked_price: Price::saturating_from_integer(200000000u128),
		}));
		assert_eq!(
			PricesModule::locked_price(EDF),
			Some(Price::saturating_from_integer(200000000u128))
		);
	});
}

#[test]
fn unlock_price_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(PricesModule::unlock_price(RuntimeOrigin::signed(5), EDF), BadOrigin);

		// unlock failed when there's no locked price
		assert_noop!(
			PricesModule::unlock_price(RuntimeOrigin::signed(1), EDF),
			Error::<Runtime>::NoLockedPrice
		);

		assert_ok!(PricesModule::lock_price(RuntimeOrigin::signed(1), EDF));
		assert_eq!(
			PricesModule::locked_price(EDF),
			Some(Price::saturating_from_integer(50000000000u128))
		);
		assert_ok!(PricesModule::unlock_price(RuntimeOrigin::signed(1), EDF));
		System::assert_last_event(RuntimeEvent::PricesModule(crate::Event::UnlockPrice {
			currency_id: EDF,
		}));
		assert_eq!(PricesModule::locked_price(EDF), None);
	});
}

#[test]
fn price_providers_work() {
	ExtBuilder::default().build().execute_with(|| {
		// issue LP
		assert_ok!(Tokens::deposit(LP_USSD_SEE, &1, 100));
		assert_eq!(Tokens::total_issuance(LP_USSD_SEE), 100);
		let lp_price_1 = lp_token_fair_price(
			Tokens::total_issuance(LP_USSD_SEE),
			MockDEX::get_liquidity_pool(USSD, SEE).0,
			MockDEX::get_liquidity_pool(USSD, SEE).1,
			PricesModule::access_price(USSD).unwrap(),
			PricesModule::access_price(SEE).unwrap(),
		);

		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_price(USSD),
			Some(Price::saturating_from_integer(1000000u128))
		);
		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_price(LSEE),
			Some(Price::saturating_from_integer(5000000000u128))
		);
		assert_eq!(RealTimePriceProvider::<Runtime>::get_price(EDF), None);
		assert_eq!(RealTimePriceProvider::<Runtime>::get_price(LP_USSD_SEE), lp_price_1);

		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(USSD),
			Some(Price::saturating_from_integer(1000000u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(LSEE),
			Some(Price::saturating_from_integer(5000000000u128))
		);
		assert_eq!(PriorityLockedPriceProvider::<Runtime>::get_price(EDF), None);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(LP_USSD_SEE),
			lp_price_1
		);

		assert_eq!(LockedPriceProvider::<Runtime>::get_price(USSD), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(LSEE), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(EDF), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(LP_USSD_SEE), None);

		// lock price
		assert_ok!(PricesModule::lock_price(RuntimeOrigin::signed(1), USSD));
		assert_ok!(PricesModule::lock_price(RuntimeOrigin::signed(1), LSEE));
		assert_noop!(
			PricesModule::lock_price(RuntimeOrigin::signed(1), EDF),
			Error::<Runtime>::AccessPriceFailed
		);
		assert_ok!(PricesModule::lock_price(RuntimeOrigin::signed(1), LP_USSD_SEE));

		assert_eq!(
			LockedPriceProvider::<Runtime>::get_price(USSD),
			Some(Price::saturating_from_integer(1000000u128))
		);
		assert_eq!(
			LockedPriceProvider::<Runtime>::get_price(LSEE),
			Some(Price::saturating_from_integer(5000000000u128))
		);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(EDF), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(LP_USSD_SEE), lp_price_1);

		// mock oracle update
		mock_oracle_update();
		let lp_price_2 = lp_token_fair_price(
			Tokens::total_issuance(LP_USSD_SEE),
			MockDEX::get_liquidity_pool(USSD, SEE).0,
			MockDEX::get_liquidity_pool(USSD, SEE).1,
			PricesModule::access_price(USSD).unwrap(),
			PricesModule::access_price(SEE).unwrap(),
		);

		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_price(USSD),
			Some(Price::saturating_from_integer(1000000u128))
		);
		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_price(LSEE),
			Some(Price::saturating_from_integer(600000000u128))
		);
		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_price(EDF),
			Some(Price::saturating_from_integer(200000000u128))
		);
		assert_eq!(RealTimePriceProvider::<Runtime>::get_price(LP_USSD_SEE), lp_price_2);

		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(USSD),
			Some(Price::saturating_from_integer(1000000u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(LSEE),
			Some(Price::saturating_from_integer(5000000000u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(EDF),
			Some(Price::saturating_from_integer(200000000u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(LP_USSD_SEE),
			lp_price_1
		);

		assert_eq!(
			LockedPriceProvider::<Runtime>::get_price(USSD),
			Some(Price::saturating_from_integer(1000000u128))
		);
		assert_eq!(
			LockedPriceProvider::<Runtime>::get_price(LSEE),
			Some(Price::saturating_from_integer(5000000000u128))
		);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(EDF), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(LP_USSD_SEE), lp_price_1);

		// unlock price
		assert_ok!(PricesModule::unlock_price(RuntimeOrigin::signed(1), USSD));
		assert_ok!(PricesModule::unlock_price(RuntimeOrigin::signed(1), LSEE));
		assert_noop!(
			PricesModule::unlock_price(RuntimeOrigin::signed(1), EDF),
			Error::<Runtime>::NoLockedPrice
		);
		assert_ok!(PricesModule::unlock_price(RuntimeOrigin::signed(1), LP_USSD_SEE));

		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(USSD),
			Some(Price::saturating_from_integer(1000000u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(LSEE),
			Some(Price::saturating_from_integer(600000000u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(EDF),
			Some(Price::saturating_from_integer(200000000u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(LP_USSD_SEE),
			lp_price_2
		);

		assert_eq!(LockedPriceProvider::<Runtime>::get_price(USSD), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(LSEE), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(EDF), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(LP_USSD_SEE), None);
	});
}
