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

//! Unit tests for the SERP Treasury module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn serplus_pool_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SerpTreasuryModule::serplus_pool(), 0);
		assert_ok!(Currencies::deposit(
			GetStableCurrencyId::get(),
			&SerpTreasuryModule::account_id(),
			500
		));
		assert_eq!(SerpTreasuryModule::serplus_pool(), 500);
	});
}

#[test]
fn total_reserve_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SerpTreasuryModule::total_reserve(BTC), 0);
		assert_ok!(Currencies::deposit(BTC, &SerpTreasuryModule::account_id(), 10));
		assert_eq!(SerpTreasuryModule::total_reserve(BTC), 10);
	});
}

#[test]
fn offset_serplus_and_standard_on_finalize_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDJ, &SerpTreasuryModule::account_id()), 0);
		assert_eq!(SerpTreasuryModule::serplus_pool(), 0);
		assert_eq!(SerpTreasuryModule::standard_pool(), 0);
		assert_eq!(Currencies::free_balance(USDJ, &SerpTreasuryModule::account_id()), 1000);
		assert_eq!(SerpTreasuryModule::serplus_pool(), 1000);
		SerpTreasuryModule::on_finalize(1);
		assert_eq!(Currencies::free_balance(USDJ, &SerpTreasuryModule::account_id()), 1000);
		assert_eq!(SerpTreasuryModule::serplus_pool(), 1000);
		assert_eq!(SerpTreasuryModule::standard_pool(), 0);
		assert_eq!(SerpTreasuryModule::standard_pool(), 300);
		SerpTreasuryModule::on_finalize(2);
		assert_eq!(Currencies::free_balance(USDJ, &SerpTreasuryModule::account_id()), 700);
		assert_eq!(SerpTreasuryModule::serplus_pool(), 700);
		assert_eq!(SerpTreasuryModule::standard_pool(), 0);
		assert_eq!(SerpTreasuryModule::standard_pool(), 800);
		SerpTreasuryModule::on_finalize(3);
		assert_eq!(Currencies::free_balance(USDJ, &SerpTreasuryModule::account_id()), 0);
		assert_eq!(SerpTreasuryModule::serplus_pool(), 0);
		assert_eq!(SerpTreasuryModule::standard_pool(), 100);
	});
}

#[test]
fn issue_standard_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 1000);
		assert_eq!(SerpTreasuryModule::standard_pool(), 0);

		assert_ok!(SerpTreasuryModule::issue_standard(&ALICE, 1000));
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 2000);
		assert_eq!(SerpTreasuryModule::standard_pool(), 0);

		assert_ok!(SerpTreasuryModule::issue_standard(&ALICE, 1000));
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 3000);
		assert_eq!(SerpTreasuryModule::standard_pool(), 1000);
	});
}

#[test]
fn burn_standard_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 1000);
		assert_eq!(SerpTreasuryModule::standard_pool(), 0);
		assert_ok!(SerpTreasuryModule::burn_standard(&ALICE, 300));
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 700);
		assert_eq!(SerpTreasuryModule::standard_pool(), 0);
	});
}

#[test]
fn issue_dexer_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SDEX, &ALICE), 1000);

		assert_ok!(SerpTreasuryModule::issue_dexer(&ALICE, 1000));
		assert_eq!(Currencies::free_balance(SDEX, &ALICE), 2000);

		assert_ok!(SerpTreasuryModule::issue_dexer(&ALICE, 1000));
		assert_eq!(Currencies::free_balance(SDEX, &ALICE), 3000);
	});
}

#[test]
fn burn_dexer_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SDEX, &ALICE), 1000);
		assert_ok!(SerpTreasuryModule::burn_dexer(&ALICE, 300));
		assert_eq!(Currencies::free_balance(SDEX, &ALICE), 700);
	});
}

#[test]
fn deposit_serplus_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(USDJ, &SerpTreasuryModule::account_id()), 0);
		assert_eq!(SerpTreasuryModule::serplus_pool(), 0);
		assert_ok!(SerpTreasuryModule::deposit_serplus(&ALICE, 300));
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 700);
		assert_eq!(Currencies::free_balance(USDJ, &SerpTreasuryModule::account_id()), 300);
		assert_eq!(SerpTreasuryModule::serplus_pool(), 300);
	});
}

#[test]
fn deposit_reserve_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SerpTreasuryModule::total_reserve(BTC), 0);
		assert_eq!(Currencies::free_balance(BTC, &SerpTreasuryModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);
		assert_eq!(SerpTreasuryModule::deposit_reserve(&ALICE, BTC, 10000).is_ok(), false);
		assert_ok!(SerpTreasuryModule::deposit_reserve(&ALICE, BTC, 500));
		assert_eq!(SerpTreasuryModule::total_reserve(BTC), 500);
		assert_eq!(Currencies::free_balance(BTC, &SerpTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 500);
	});
}

#[test]
fn withdraw_reserve_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SerpTreasuryModule::deposit_reserve(&ALICE, BTC, 500));
		assert_eq!(SerpTreasuryModule::total_reserve(BTC), 500);
		assert_eq!(Currencies::free_balance(BTC, &SerpTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(BTC, &BOB), 1000);
		assert_eq!(SerpTreasuryModule::withdraw_reserve(&BOB, BTC, 501).is_ok(), false);
		assert_ok!(SerpTreasuryModule::withdraw_reserve(&BOB, BTC, 400));
		assert_eq!(SerpTreasuryModule::total_reserve(BTC), 100);
		assert_eq!(Currencies::free_balance(BTC, &SerpTreasuryModule::account_id()), 100);
		assert_eq!(Currencies::free_balance(BTC, &BOB), 1400);
	});
}

#[test]
fn get_total_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SerpTreasuryModule::deposit_reserve(&ALICE, BTC, 500));
		assert_eq!(SerpTreasuryModule::get_total_setter(BTC), 500);
	});
}

#[test]
fn get_standard_proportion_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SerpTreasuryModule::get_standard_proportion(100),
			Ratio::saturating_from_rational(100, Currencies::total_issuance(USDJ))
		);
	});
}

#[test]
fn auction_serplus_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(SerpTreasuryModule::auction_serplus(Origin::signed(5), 100, USDJ), BadOrigin,);
		assert_noop!(
			SerpTreasuryModule::auction_serplus(Origin::signed(1), 100, USDJ),
			Error::<Runtime>::SerplusPoolNotEnough,
		);
		assert_eq!(TOTAL_SERPLUS_IN_AUCTION.with(|v| *v.borrow_mut()), 0);
		assert_ok!(SerpTreasuryModule::auction_serplus(Origin::signed(1), 100, USDJ));
		assert_eq!(TOTAL_SERPLUS_IN_AUCTION.with(|v| *v.borrow_mut()), 1);
	});
}
