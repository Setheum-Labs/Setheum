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
use mock::*;

#[test]
fn issue_standard_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 1000);

		assert_ok!(SerpTreasuryModule::issue_standard(USDJ, &ALICE, 1000));
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 2000);

		assert_ok!(SerpTreasuryModule::issue_standard(USDJ, &ALICE, 1000));
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 3000);
	});
}

#[test]
fn burn_standard_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 1000);
		assert_ok!(SerpTreasuryModule::burn_standard(USDJ, &ALICE, 300));
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 700);
	});
}

#[test]
fn issue_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 1000);

		assert_ok!(SerpTreasuryModule::issue_setter(&ALICE, 1000));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 2000);

		assert_ok!(SerpTreasuryModule::issue_setter(&ALICE, 1000));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 3000);
	});
}

#[test]
fn burn_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 1000);
		assert_ok!(SerpTreasuryModule::burn_setter(&ALICE, 300));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 700);
	});
}

#[test]
fn deposit_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETT, &SerpTreasuryModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 1000);
		assert_eq!(SerpTreasuryModule::deposit_setter(&ALICE, 10000).is_ok(), false);
		assert_ok!(SerpTreasuryModule::deposit_setter(&ALICE, 500));
		assert_eq!(Currencies::free_balance(SETT, &SerpTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 500);
	});
}

#[test]
fn swap_dinar_to_exact_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Currencies::deposit(SETT, &ALICE, 10000));
		assert_ok!(Currencies::deposit(DNAR, &ALICE, 10000));
		assert_ok!(SetheumDEX::add_liquidity(
			Origin::signed(ALICE),
			SETT,
			DNAR,
			900,
			1000,
			0,
		));
		assert_ok!(Currencies::deposit(DNAR, &SerpTreasuryModule::account_id(), 10000));
		assert_ok!(Currencies::deposit(SETT, &SerpTreasuryModule::account_id(), 10000));
		assert_ok!(SerpTreasuryModule::swap_dinar_to_exact_setter(
			10, None
		));
		assert_eq!(Currencies::free_balance(SETT, &SerpTreasuryModule::account_id()), 10000);
	});
}

#[test]
fn swap_exact_setter_to_dinar_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Currencies::deposit(SETT, &ALICE, 10000));
		assert_ok!(Currencies::deposit(DNAR, &ALICE, 10000));
		assert_ok!(SetheumDEX::add_liquidity(
			Origin::signed(ALICE),
			DNAR,
			SETT,
			900,
			1000,
			0,
		));
		assert_ok!(Currencies::deposit(SETT, &SerpTreasuryModule::account_id(), 10000));
		assert_ok!(SerpTreasuryModule::swap_exact_setter_to_dinar(
			100, None
		));
		assert_eq!(Currencies::free_balance(SETT, &SerpTreasuryModule::account_id()), 10000);
	});
}
#[test]
fn swap_exact_settcurrency_to_dinar_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Currencies::deposit(USDJ, &BOB, 10000));
		assert_ok!(Currencies::deposit(DNAR, &BOB, 10000));
		assert_ok!(SetheumDEX::add_liquidity(
			Origin::signed(BOB),
			USDJ,
			DNAR,
			1000,
			1000,
			0,
		));
		assert_ok!(Currencies::deposit(USDJ, &SerpTreasuryModule::account_id(), 10000));
		assert_ok!(SerpTreasuryModule::swap_exact_settcurrency_to_dinar(
			USDJ, 100, None
		));
		assert_eq!(Currencies::free_balance(USDJ, &SerpTreasuryModule::account_id()), 100);

		assert_noop!(
			SerpTreasuryModule::swap_exact_settcurrency_to_dinar(USDJ, 100, Some(&vec![USDJ, DNAR])),
			Error::<Runtime>::InvalidSwapPath
		);
	});
}