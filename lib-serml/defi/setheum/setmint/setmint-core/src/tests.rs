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

//! Unit tests for the setmint module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use orml_traits::Change;
use sp_runtime::FixedPointNumber;
use support::{Rate, Ratio};

#[test]
fn authorize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_eq!(PalletBalances::reserved_balance(ALICE), 0);
		assert_ok!(SerpSetmint::authorize(Origin::signed(ALICE), SERP, BOB));
		assert_eq!(PalletBalances::reserved_balance(ALICE), DepositPerAuthorization::get());
		System::assert_last_event(Event::SerpSetmint(crate::Event::Authorization {
			authorizer: ALICE,
			authorizee: BOB,
			collateral_type: SERP,
		}));
		assert_ok!(SerpSetmint::check_authorization(&ALICE, &BOB, SERP));
		assert_noop!(
			SerpSetmint::authorize(Origin::signed(ALICE), SERP, BOB),
			Error::<Runtime>::AlreadyAuthorized
		);
	});
}

#[test]
fn unauthorize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SerpSetmint::authorize(Origin::signed(ALICE), SERP, BOB));
		assert_eq!(PalletBalances::reserved_balance(ALICE), 100);
		assert_ok!(SerpSetmint::check_authorization(&ALICE, &BOB, SERP));

		assert_ok!(SerpSetmint::unauthorize(Origin::signed(ALICE), SERP, BOB));
		assert_eq!(PalletBalances::reserved_balance(ALICE), 0);
		System::assert_last_event(Event::SerpSetmint(crate::Event::UnAuthorization {
			authorizer: ALICE,
			authorizee: BOB,
			collateral_type: SERP,
		}));
			assert_noop!(
			SerpSetmint::check_authorization(&ALICE, &BOB, SERP),
			Error::<Runtime>::NoPermission
		);
		assert_noop!(
			SerpSetmint::unauthorize(Origin::signed(ALICE), SERP, BOB),
			Error::<Runtime>::AuthorizationNotExists
		);
	});
}

#[test]
fn unauthorize_all_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SerpSetmint::authorize(Origin::signed(ALICE), SERP, BOB));
		assert_ok!(SerpSetmint::authorize(Origin::signed(ALICE), DNAR, CAROL));
		assert_eq!(PalletBalances::reserved_balance(ALICE), 200);
		assert_ok!(SerpSetmint::unauthorize_all(Origin::signed(ALICE)));
		assert_eq!(PalletBalances::reserved_balance(ALICE), 0);
		System::assert_last_event(Event::SerpSetmint(crate::Event::UnAuthorizationAll {
			authorizer: ALICE,
		}));

		assert_noop!(
			SerpSetmint::check_authorization(&ALICE, &BOB, SERP),
			Error::<Runtime>::NoPermission
		);
		assert_noop!(
			SerpSetmint::check_authorization(&ALICE, &BOB, DNAR),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn transfer_loan_from_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SerpSetmint::adjust_loan(Origin::signed(ALICE), SERP, 100, 50));
		assert_ok!(SerpSetmint::authorize(Origin::signed(ALICE), SERP, BOB));
		assert_ok!(SerpSetmint::transfer_loan_from(Origin::signed(BOB), SERP, ALICE));
		assert_eq!(LoansModule::positions(SERP, BOB).collateral, 100);
		assert_eq!(LoansModule::positions(SERP, BOB).debit, 50);
	});
}

#[test]
fn transfer_unauthorization_loans_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			SerpSetmint::transfer_loan_from(Origin::signed(ALICE), SERP, BOB),
			Error::<Runtime>::NoPermission,
		);
	});
}

#[test]
fn adjust_loan_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SerpSetmint::adjust_loan(Origin::signed(ALICE), SERP, 100, 50));
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 50);
	});
}

#[test]
fn on_emergency_shutdown_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		mock_shutdown();
		assert_noop!(
			SerpSetmint::adjust_loan(Origin::signed(ALICE), SERP, 100, 50),
			Error::<Runtime>::AlreadyShutdown,
		);
		assert_noop!(
			SerpSetmint::transfer_loan_from(Origin::signed(ALICE), SERP, BOB),
			Error::<Runtime>::AlreadyShutdown,
		);
		assert_noop!(
			SerpSetmint::close_loan_has_debit_by_dex(Origin::signed(ALICE), SERP, 100),
			Error::<Runtime>::AlreadyShutdown,
		);
	});
}

#[test]
fn close_loan_has_debit_by_dex_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SerpSetmint::adjust_loan(Origin::signed(ALICE), SERP, 100, 50));
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 50);

		assert_ok!(SerpSetmint::close_loan_has_debit_by_dex(
			Origin::signed(ALICE),
			SERP,
			100,
		));
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
	});
}
