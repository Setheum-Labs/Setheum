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
use frame_support::assert_ok;
use mock::*;

#[test]
fn issue_standard_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 1000);

		assert_ok!(SerpTreasuryModule::issue_standard(SETUSD, &ALICE, 1000));
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 2000);

		assert_ok!(SerpTreasuryModule::issue_standard(SETUSD, &ALICE, 1000));
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 3000);
	});
}

#[test]
fn burn_standard_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 1000);
		assert_ok!(SerpTreasuryModule::burn_standard(SETUSD, &ALICE, 300));
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 700);
	});
}

#[test]
fn issue_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 1000);

		assert_ok!(SerpTreasuryModule::issue_setter(&ALICE, 1000));
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 2000);

		assert_ok!(SerpTreasuryModule::issue_setter(&ALICE, 1000));
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 3000);
	});
}

#[test]
fn burn_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 1000);
		assert_ok!(SerpTreasuryModule::burn_setter(&ALICE, 300));
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 700);
	});
}

#[test]
fn deposit_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETR, &SerpTreasuryModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 1000);
		assert_eq!(SerpTreasuryModule::deposit_setter(&ALICE, 10000).is_ok(), false);
		assert_ok!(SerpTreasuryModule::deposit_setter(&ALICE, 500));
		assert_eq!(Currencies::free_balance(SETR, &SerpTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 500);
	});
}

#[test]
fn on_serpdown_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Currencies::deposit(SETR, &ALICE, 10000));
		assert_ok!(Currencies::deposit(DNAR, &ALICE, 10000));
		assert_ok!(SetheumDEX::add_liquidity(
			Origin::signed(ALICE),
			SETR,
			DNAR,
			1000,
			100,
			0,
		));
		assert_ok!(Currencies::deposit(DNAR, &SerpTreasuryModule::account_id(), 10000));
		assert_ok!(Currencies::deposit(SETR, &SerpTreasuryModule::account_id(), 10000));
		assert_ok!(SerpTreasuryModule::on_serpdown(SETR, 10));
		assert_eq!(Currencies::free_balance(SETR, &SerpTreasuryModule::account_id()), 10020);
		assert_eq!(Currencies::free_balance(DNAR, &SerpTreasuryModule::account_id()), 10000);
	});
}

#[test]
fn on_serpup_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Currencies::deposit(SETR, &ALICE, 10000));
		assert_ok!(Currencies::deposit(DNAR, &ALICE, 10000));
		assert_ok!(SetheumDEX::add_liquidity(
			Origin::signed(ALICE),
			SETR,
			DNAR,
			1000,
			100,
			0,
		));
		assert_ok!(Currencies::deposit(DNAR, &SerpTreasuryModule::account_id(), 10000));
		assert_ok!(Currencies::deposit(SETR, &SerpTreasuryModule::account_id(), 10000));
		assert_ok!(SerpTreasuryModule::on_serpup(SETR, 10));
		assert_eq!(Currencies::free_balance(SETR, &SerpTreasuryModule::account_id()), 10004);
		assert_eq!(Currencies::free_balance(SETR, &VAULT), 1000);
		assert_eq!(Currencies::free_balance(SETR, &CHARITY_FUND), 1001);
		assert_eq!(Currencies::free_balance(DNAR, &SerpTreasuryModule::account_id()), 10000);
	});
}
