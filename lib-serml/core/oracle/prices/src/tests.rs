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

//! Unit tests for the prices module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
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
fn access_price_of_stable_currency() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			PricesModule::access_price(USDI),
			Some(Price::saturating_from_integer(1u128))
		);

		mock_oracle_update();
		assert_eq!(
			PricesModule::access_price(USDI),
			Some(Price::saturating_from_integer(1u128))
		);
	});
}

#[test]
fn access_price_of_dex_share_currency() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			PricesModule::access_price(WBTC),
			Some(Price::saturating_from_integer(100u128))
		); // 100 USD, right shift the decimal point (18-12) places
		assert_eq!(
			PricesModule::access_price(USDI),
			Some(Price::saturating_from_integer(1u128))
		);
		assert_eq!(Tokens::total_issuance(LP_USDI_WBTC), 0);
		assert_eq!(MockDEX::get_liquidity_pool(USDI, WBTC), (10000, 200));

		// when the total issuance of dex share currency is zero
		assert_eq!(PricesModule::access_price(LP_USDI_WBTC), None);

		// issue LP
		assert_ok!(Tokens::deposit(LP_USDI_WBTC, &1, 100));
		assert_eq!(Tokens::total_issuance(LP_USDI_WBTC), 100);

		let lp_price_1 = lp_token_fair_price(
			Tokens::total_issuance(LP_USDI_WBTC),
			MockDEX::get_liquidity_pool(USDI, WBTC).0,
			MockDEX::get_liquidity_pool(USDI, WBTC).1,
			PricesModule::access_price(USDI).unwrap(),
			PricesModule::access_price(WBTC).unwrap(),
		);
		assert_eq!(PricesModule::access_price(LP_USDI_WBTC), lp_price_1);

		// issue more LP
		assert_ok!(Tokens::deposit(LP_USDI_WBTC, &1, 100));
		assert_eq!(Tokens::total_issuance(LP_USDI_WBTC), 200);

		let lp_price_2 = lp_token_fair_price(
			Tokens::total_issuance(LP_USDI_WBTC),
			MockDEX::get_liquidity_pool(USDI, WBTC).0,
			MockDEX::get_liquidity_pool(USDI, WBTC).1,
			PricesModule::access_price(USDI).unwrap(),
			PricesModule::access_price(WBTC).unwrap(),
		);
		assert_eq!(PricesModule::access_price(LP_USDI_WBTC), lp_price_2);

		mock_oracle_update();

		let lp_price_3 = lp_token_fair_price(
			Tokens::total_issuance(LP_USDI_WBTC),
			MockDEX::get_liquidity_pool(USDI, WBTC).0,
			MockDEX::get_liquidity_pool(USDI, WBTC).1,
			PricesModule::access_price(USDI).unwrap(),
			PricesModule::access_price(WBTC).unwrap(),
		);
		assert_eq!(PricesModule::access_price(LP_USDI_WBTC), lp_price_3);
	});
}

#[test]
fn access_price_of_other_currency() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(PricesModule::access_price(SETM), Some(Price::saturating_from_integer(0)));
		assert_eq!(PricesModule::access_price(USDT), Some(Price::saturating_from_rational(1, 4)));

		mock_oracle_update();

		assert_eq!(
			PricesModule::access_price(SETM),
			Some(Price::saturating_from_integer(30u128))
		); // 30 USD, right shift the decimal point (18-12) places
		assert_eq!(
			PricesModule::access_price(ETH),
			Some(Price::saturating_from_integer(40000u128))
		); // 200 USD, right shift the decimal point (18-12) places
	});
}

#[test]
fn lock_price_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(PricesModule::unlock_price(Origin::signed(5), ETH), BadOrigin);

		// lock the price of ETH
		assert_eq!(
			PricesModule::access_price(ETH),
			Some(Price::saturating_from_integer(50000u128))
		);
		assert_eq!(PricesModule::locked_price(ETH), None);
		assert_ok!(PricesModule::lock_price(Origin::signed(1), ETH));
		System::assert_last_event(Event::PricesModule(crate::Event::LockPrice(
			ETH,
			Price::saturating_from_integer(50000u128),
		)));
		assert_eq!(
			PricesModule::locked_price(ETH),
			Some(Price::saturating_from_integer(50000u128))
		);

		// lock the price of USDT when the price of USDT from oracle is some
		assert_eq!(
			PricesModule::access_price(USDT),
			Some(Price::saturating_from_rational(1, 4))
		);
		assert_eq!(PricesModule::locked_price(USDT), None);
		assert_ok!(PricesModule::lock_price(Origin::signed(1), USDT));
		System::assert_last_event(Event::PricesModule(crate::Event::LockPrice(
			USDT,
			Price::saturating_from_rational(1, 4),
		)));
		assert_eq!(
			PricesModule::locked_price(USDT),
			Some(Price::saturating_from_rational(1, 4))
		);
	});
}

#[test]
fn unlock_price_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(PricesModule::unlock_price(Origin::signed(5), ETH), BadOrigin);

		// unlock failed when there's no locked price
		assert_noop!(
			PricesModule::unlock_price(Origin::signed(1), ETH),
			Error::<Runtime>::NoLockedPrice
		);

		assert_ok!(PricesModule::lock_price(Origin::signed(1), ETH));
		assert_eq!(
			PricesModule::locked_price(ETH),
			Some(Price::saturating_from_integer(50000u128))
		);
		assert_ok!(PricesModule::unlock_price(Origin::signed(1), ETH));
		System::assert_last_event(Event::PricesModule(crate::Event::UnlockPrice(ETH)));
		assert_eq!(PricesModule::locked_price(ETH), None);
	});
}

#[test]
fn price_providers_work() {
	ExtBuilder::default().build().execute_with(|| {
		// issue LP
		assert_ok!(Tokens::deposit(LP_USDI_WBTC, &1, 100));
		assert_eq!(Tokens::total_issuance(LP_USDI_WBTC), 100);
		let lp_price_1 = lp_token_fair_price(
			Tokens::total_issuance(LP_USDI_WBTC),
			MockDEX::get_liquidity_pool(USDI, WBTC).0,
			MockDEX::get_liquidity_pool(USDI, WBTC).1,
			PricesModule::access_price(USDI).unwrap(),
			PricesModule::access_price(WBTC).unwrap(),
		);

		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_price(USDI),
			Some(Price::saturating_from_integer(1u128))
		);
		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_price(ETH),
			Some(Price::saturating_from_integer(50000u128))
		);
		assert_eq!(RealTimePriceProvider::<Runtime>::get_price(USDT), Some(Price::saturating_from_rational(1, 4)));
		assert_eq!(RealTimePriceProvider::<Runtime>::get_price(LP_USDI_WBTC), lp_price_1);
		assert_eq!(RealTimePriceProvider::<Runtime>::get_relative_price(ETH, USDT), Some(Price::saturating_from_integer(200_000)));

		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(USDI),
			Some(Price::saturating_from_integer(1u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(ETH),
			Some(Price::saturating_from_integer(50000u128))
		);
		assert_eq!(PriorityLockedPriceProvider::<Runtime>::get_price(USDT), Some(Price::saturating_from_rational(1, 4)));
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(LP_USDI_WBTC),
			lp_price_1
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_relative_price(ETH, USDT),
			Some(Price::saturating_from_integer(200_000))
		);

		assert_eq!(LockedPriceProvider::<Runtime>::get_price(USDI), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(ETH), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(USDT), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(LP_USDI_WBTC), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_relative_price(ETH, USDT), None);

		// lock price
		assert_ok!(PricesModule::lock_price(Origin::signed(1), USDI));
		assert_ok!(PricesModule::lock_price(Origin::signed(1), ETH));
	
		assert_ok!(PricesModule::lock_price(Origin::signed(1), LP_USDI_WBTC));

		assert_eq!(
			LockedPriceProvider::<Runtime>::get_price(USDI),
			Some(Price::saturating_from_integer(1u128))
		);
		assert_eq!(
			LockedPriceProvider::<Runtime>::get_price(ETH),
			Some(Price::saturating_from_integer(50000u128))
		);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(USDT), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(LP_USDI_WBTC), lp_price_1);
		assert_eq!(LockedPriceProvider::<Runtime>::get_relative_price(ETH, USDT), None);

		// mock oracle update
		mock_oracle_update();
		let lp_price_2 = lp_token_fair_price(
			Tokens::total_issuance(LP_USDI_WBTC),
			MockDEX::get_liquidity_pool(USDI, WBTC).0,
			MockDEX::get_liquidity_pool(USDI, WBTC).1,
			PricesModule::access_price(USDI).unwrap(),
			PricesModule::access_price(WBTC).unwrap(),
		);

		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_price(USDI),
			Some(Price::saturating_from_integer(1u128))
		);
		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_price(ETH),
			Some(Price::saturating_from_integer(40000u128))
		);
		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_price(USDT),
			Some(Price::saturating_from_rational(1, 4))
		);
		assert_eq!(RealTimePriceProvider::<Runtime>::get_price(LP_USDI_WBTC), lp_price_2);
		assert_eq!(
			RealTimePriceProvider::<Runtime>::get_relative_price(ETH, USDT),
			Some(Price::saturating_from_integer(160_000u128))
		);

		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(USDI),
			Some(Price::saturating_from_integer(1u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(ETH),
			Some(Price::saturating_from_integer(50000u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(USDT),
			Some(Price::saturating_from_rational(1, 4))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(LP_USDI_WBTC),
			lp_price_1
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_relative_price(ETH, USDT),
			Some(Price::saturating_from_integer(200_000u128))
		);

		assert_eq!(
			LockedPriceProvider::<Runtime>::get_price(USDI),
			Some(Price::saturating_from_integer(1u128))
		);
		assert_eq!(
			LockedPriceProvider::<Runtime>::get_price(ETH),
			Some(Price::saturating_from_integer(50000u128))
		);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(USDT), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(LP_USDI_WBTC), lp_price_1);
		assert_eq!(LockedPriceProvider::<Runtime>::get_relative_price(ETH, USDT), None);

		// unlock price
		assert_ok!(PricesModule::unlock_price(Origin::signed(1), USDI));
		assert_ok!(PricesModule::unlock_price(Origin::signed(1), ETH));
		assert_noop!(
			PricesModule::unlock_price(Origin::signed(1), USDT),
			Error::<Runtime>::NoLockedPrice
		);
		assert_ok!(PricesModule::unlock_price(Origin::signed(1), LP_USDI_WBTC));

		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(USDI),
			Some(Price::saturating_from_integer(1u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(ETH),
			Some(Price::saturating_from_integer(40000u128))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(USDT),
			Some(Price::saturating_from_rational(1, 4))
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_price(LP_USDI_WBTC),
			lp_price_2
		);
		assert_eq!(
			PriorityLockedPriceProvider::<Runtime>::get_relative_price(ETH, USDT),
			Some(Price::saturating_from_integer(160_000u128))
		);

		assert_eq!(LockedPriceProvider::<Runtime>::get_price(USDI), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(ETH), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(USDT), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_price(LP_USDI_WBTC), None);
		assert_eq!(LockedPriceProvider::<Runtime>::get_relative_price(ETH, USDT), None);
	});
}
