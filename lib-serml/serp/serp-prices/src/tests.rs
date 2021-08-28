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
	FixedPointNumber
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
fn get_price_from_oracle() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SerpPrices::get_price(BTC),
			Some(Price::saturating_from_integer(500000000000000u128))
		); // 50000 USD, right shift the decimal point (18-8) places
		assert_eq!(
			SerpPrices::get_price(DNAR),
			Some(Price::saturating_from_integer(100000000u128))
		); // 100 USD, right shift the decimal point (18-12) places
	});
}

#[test]
fn get_price_of_stable_currency_id() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SerpPrices::get_price(SETUSD),
			Some(Price::saturating_from_integer(1000000u128))
		); // 1 USD, right shift the decimal point (18-12) places
	});
}

#[test]
fn get_price_of_lp_token_currency_id() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(MockDEX::get_liquidity_pool(SETUSD, DRAM), (0, 0));
		assert_eq!(SerpPrices::get_price(LP_SETUSD_DRAM), None);
		assert_ok!(Tokens::deposit(LP_SETUSD_DRAM, &1, 100));
		assert_eq!(Tokens::total_issuance(LP_SETUSD_DRAM), 100);
		assert_eq!(
			SerpPrices::get_price(SETUSD),
			Some(Price::saturating_from_rational(1000000u128, 1))
		);
		assert_eq!(
			SerpPrices::get_price(LP_SETUSD_DRAM),
			lp_token_fair_price(
				Tokens::total_issuance(LP_SETUSD_DRAM),
				MockDEX::get_liquidity_pool(SETUSD, DRAM).0,
				MockDEX::get_liquidity_pool(SETUSD, DRAM).1,
				SerpPrices::get_price(SETUSD).unwrap(),
				SerpPrices::get_price(DRAM).unwrap()
			)
		);

		assert_eq!(MockDEX::get_liquidity_pool(BTC, SETUSD), (0, 0));
		assert_eq!(SerpPrices::get_price(LP_BTC_SETUSD), None);
	});
}

#[test]
fn get_relative_price_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SerpPrices::get_relative_price(BTC, DNAR),
			Some(Price::saturating_from_rational(5000000, 1)) /* 1DNAR = 100SETUSD, right shift the decimal point (12-10)
			                                                 * places */
		);
	});
}

#[test]
fn lock_price_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SerpPrices::get_price(BTC),
			Some(Price::saturating_from_integer(500000000000000u128))
		);
		LockedPrice::<Runtime>::insert(BTC, Price::saturating_from_integer(80000));
		assert_eq!(
			SerpPrices::get_price(BTC),
			Some(Price::saturating_from_integer(800000000000000u128))
		);
	});
}

#[test]
fn lock_price_call_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(SerpPrices::lock_price(Origin::signed(5), BTC), BadOrigin,);
		assert_ok!(SerpPrices::lock_price(Origin::signed(1), BTC));
		System::assert_last_event(Event::SerpPrices(crate::Event::LockPrice(
			BTC,
			Price::saturating_from_integer(50000)
		)));
		assert_eq!(
			SerpPrices::locked_price(BTC),
			Some(Price::saturating_from_integer(50000))
		);
	});
}

#[test]
fn unlock_price_call_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		LockedPrice::<Runtime>::insert(BTC, Price::saturating_from_integer(80000));
		assert_noop!(SerpPrices::unlock_price(Origin::signed(5), BTC), BadOrigin,);
		assert_ok!(SerpPrices::unlock_price(Origin::signed(1), BTC));
		System::assert_last_event(Event::SerpPrices(crate::Event::UnlockPrice(BTC)));
		assert_eq!(SerpPrices::locked_price(BTC), None);
	});
}
