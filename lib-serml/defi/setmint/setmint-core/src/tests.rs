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
		assert_ok!(Setmint::authorize(Origin::signed(ALICE), ETH, BOB));
		assert_eq!(PalletBalances::reserved_balance(ALICE), DepositPerAuthorization::get());
		System::assert_last_event(Event::Setmint(crate::Event::Authorization {
			authorizer: ALICE,
			authorizee: BOB,
			collateral_type: ETH,
		}));
		assert_ok!(Setmint::check_authorization(&ALICE, &BOB, ETH));
		assert_noop!(
			Setmint::authorize(Origin::signed(ALICE), ETH, BOB),
			Error::<Runtime>::AlreadyAuthorized
		);
	});
}

#[test]
fn unauthorize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Setmint::authorize(Origin::signed(ALICE), ETH, BOB));
		assert_eq!(PalletBalances::reserved_balance(ALICE), 100);
		assert_ok!(Setmint::check_authorization(&ALICE, &BOB, ETH));

		assert_ok!(Setmint::unauthorize(Origin::signed(ALICE), ETH, BOB));
		assert_eq!(PalletBalances::reserved_balance(ALICE), 0);
		System::assert_last_event(Event::Setmint(crate::Event::UnAuthorization {
			authorizer: ALICE,
			authorizee: BOB,
			collateral_type: ETH,
		}));
			assert_noop!(
			Setmint::check_authorization(&ALICE, &BOB, ETH),
			Error::<Runtime>::NoPermission
		);
		assert_noop!(
			Setmint::unauthorize(Origin::signed(ALICE), ETH, BOB),
			Error::<Runtime>::AuthorizationNotExists
		);
	});
}

#[test]
fn unauthorize_all_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Setmint::authorize(Origin::signed(ALICE), ETH, BOB));
		assert_ok!(Setmint::authorize(Origin::signed(ALICE), WBTC, CAROL));
		assert_eq!(PalletBalances::reserved_balance(ALICE), 200);
		assert_ok!(Setmint::unauthorize_all(Origin::signed(ALICE)));
		assert_eq!(PalletBalances::reserved_balance(ALICE), 0);
		System::assert_last_event(Event::Setmint(crate::Event::UnAuthorizationAll {
			authorizer: ALICE,
		}));

		assert_noop!(
			Setmint::check_authorization(&ALICE, &BOB, ETH),
			Error::<Runtime>::NoPermission
		);
		assert_noop!(
			Setmint::check_authorization(&ALICE, &BOB, WBTC),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn transfer_loan_from_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			ETH,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(Setmint::adjust_loan(Origin::signed(ALICE), ETH, 100, 50));
		assert_ok!(Setmint::authorize(Origin::signed(ALICE), ETH, BOB));
		assert_ok!(Setmint::transfer_loan_from(Origin::signed(BOB), ETH, ALICE));
		assert_eq!(LoansModule::positions(ETH, BOB).collateral, 100);
		assert_eq!(LoansModule::positions(ETH, BOB).debit, 50);
	});
}

#[test]
fn transfer_unauthorization_loans_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Setmint::transfer_loan_from(Origin::signed(ALICE), ETH, BOB),
			Error::<Runtime>::NoPermission,
		);
	});
}

#[test]
fn adjust_loan_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			ETH,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(Setmint::adjust_loan(Origin::signed(ALICE), ETH, 100, 50));
		assert_eq!(LoansModule::positions(ETH, ALICE).collateral, 100);
		assert_eq!(LoansModule::positions(ETH, ALICE).debit, 50);
	});
}

#[test]
fn on_emergency_shutdown_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		mock_shutdown();
		assert_noop!(
			Setmint::adjust_loan(Origin::signed(ALICE), ETH, 100, 50),
			Error::<Runtime>::AlreadyShutdown,
		);
		assert_noop!(
			Setmint::transfer_loan_from(Origin::signed(ALICE), ETH, BOB),
			Error::<Runtime>::AlreadyShutdown,
		);
		assert_noop!(
			Setmint::close_loan_has_debit_by_dex(Origin::signed(ALICE), ETH, 100),
			Error::<Runtime>::AlreadyShutdown,
		);
	});
}

#[test]
fn close_loan_has_debit_by_dex_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			ETH,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(Setmint::adjust_loan(Origin::signed(ALICE), ETH, 100, 50));
		assert_eq!(LoansModule::positions(ETH, ALICE).collateral, 100);
		assert_eq!(LoansModule::positions(ETH, ALICE).debit, 50);

		assert_ok!(Setmint::close_loan_has_debit_by_dex(
			Origin::signed(ALICE),
			ETH,
			100,
		));
		assert_eq!(LoansModule::positions(ETH, ALICE).collateral, 0);
		assert_eq!(LoansModule::positions(ETH, ALICE).debit, 0);
	});
}
