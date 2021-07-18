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

//! Unit tests for the SERP Treasury module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn cashdrop_rate_works() {
	ExtBuilder::default().build().execute_with(|| {
		// set the cashdrop_rate of USDJ to 5%.
		assert_ok!(SettPay::update_cashdrop_rate(Origin::signed(ALICE), USDJ, 5, 100));
		// get cashdrop_rate.
		assert_eq!(SettPay::get_cashdrop_rate(USDJ), 5, 100);
	});
}

#[test]
fn minimum_claimable_transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		// set the minimum_claimable_transfer of USDJ to 20,
		assert_ok!(SettPay::update_minimum_claimable_transfer(Origin::signed(ALICE), USDJ, 20));
		// get minimum_claimable_transfer.
		assert_eq!(SettPay::get_minimum_claimable_transfer(USDJ), 20);
	});
}

#[test]
fn claim_setter_cashdrop_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettPay::update_cashdrop_rate(Origin::signed(ALICE), SETT, 5, 100));
		assert_eq!(SettPay::get_cashdrop_rate(SETT), 5, 100);

		assert_eq!(Currencies::free_balance(SETT, &ALICE), 100_000);
		assert_eq!(Currencies::free_balance(SETT, &BOB), 100_000);
		assert_eq!(Currencies::free_balance(SETT, &SETTPAY_TREASURY), 100_000);

		assert_ok!(Currencies::transfer(&ALICE, &BOB, SETT, 10_000, true));
		assert_ok!(SettPay::claim_cashdrop(SETT, &ALICE, 10_000));

		assert_eq!(Currencies::free_balance(SETT, &SETTPAY_TREASURY), 99_500);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 90_500);
		assert_eq!(Currencies::free_balance(SETT, &BOB), 110_000);
	});
}

#[test]
fn claim_settcurrency_cashdrop_works() {
	ExtBuilder::default().build().execute_with(|| {
		// uses the default cashdrop rate of 2%
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 100_000);
		assert_eq!(Currencies::free_balance(USDJ, &BOB), 100_000);
		assert_eq!(Currencies::free_balance(USDJ, &SETTPAY_TREASURY), 100_000);

		assert_ok!(Currencies::transfer(&ALICE, &BOB, USDJ, 10_000, true));
		assert_ok!(SettPay::claim_cashdrop(USDJ, &ALICE, 10_000));

		assert_eq!(Currencies::free_balance(USDJ, &SETTPAY_TREASURY), 99_800);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 90_200);
		assert_eq!(Currencies::free_balance(USDJ, &BOB), 110_000);
	});
}

#[test]
fn claim_native_cashdrop_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettPay::update_cashdrop_rate(Origin::signed(ALICE), DNAR, 5, 100));
		assert_eq!(SettPay::get_cashdrop_rate(DNAR), 5, 100);

		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 100_000);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 100_000);
		assert_eq!(Currencies::free_balance(DNAR, &BOB), 100_000);
		assert_eq!(Currencies::free_balance(DNAR, &SETTPAY_TREASURY), 100_000);

		assert_eq!(
			SetheumPrices::get_relative_price(DNAR, SETT),
			Some(Price::saturating_from_rational(10000, 1)) /* 1DNAR = 100SETT, right shift the decimal point (12-10)
			                                                 * places */
		);

		assert_ok!(Currencies::transfer(&ALICE, &BOB, DNAR, 10_000, true));
		assert_ok!(SettPay::claim_cashdrop(DNAR, &ALICE, 10_000));

		assert_eq!(Currencies::free_balance(DNAR, &SETTPAY_TREASURY), 100_000);
		assert_eq!(Currencies::free_balance(SETT, &SETTPAY_TREASURY), 50_000);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 150_000);
		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 90_000);
		assert_eq!(Currencies::free_balance(DNAR, &BOB), 110_000);
	});
}

#[test]
fn claim_dexer_cashdrop_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettPay::update_cashdrop_rate(Origin::signed(ALICE), DRAM, 5, 100));
		assert_eq!(SettPay::get_cashdrop_rate(DRAM), 5, 100);

		assert_eq!(Currencies::free_balance(DRAM, &ALICE), 100_000);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 100_000);
		assert_eq!(Currencies::free_balance(DRAM, &BOB), 100_000);
		assert_eq!(Currencies::free_balance(DRAM, &SETTPAY_TREASURY), 100_000);

		assert_eq!(DRAM
			SetheumPrices::get_relative_price(DRAM, SETT),
			Some(Price::saturating_from_rational(10000, 1)) /* 1DRAM = 100SETT, right shift the decimal point (12-10)
			                                                 * places */
		);
		
		assert_ok!(Currencies::transfer(&ALICE, &BOB, DRAM, 10_000, true));
		assert_ok!(SettPay::claim_cashdrop(DRAM, &ALICE, 10_000));

		assert_eq!(Currencies::free_balance(DRAM, &SETTPAY_TREASURY), 100_000);
		assert_eq!(Currencies::free_balance(SETT, &SETTPAY_TREASURY), 50_000);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 150_000);
		assert_eq!(Currencies::free_balance(DRAM, &ALICE), 90_000);
		assert_eq!(Currencies::free_balance(DRAM, &BOB), 110_000);
	});
}

#[test]
fn deposit_setter_drop_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 100_000);

		assert_ok!(SettPay::deposit_setter_drop(&ALICE, 2_000));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 102_000);
	});
}

#[test]
fn deposit_settcurrency_drop_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 100_000);

		assert_ok!(SettPay::deposit_settcurrency_drop(&ALICE, 2_000));
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 102_000);
	});
}
