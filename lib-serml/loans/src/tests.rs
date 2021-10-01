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

//! Unit tests for the loans module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

#[test]
fn debits_key() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(LoansModule::setdollar_positions(BTC, &ALICE).debit, 0);
		assert_ok!(LoansModule::adjust_position(&ALICE, BTC, SETUSD, 200, 200));
		assert_eq!(LoansModule::setdollar_positions(BTC, &ALICE).debit, 200);
		assert_eq!(Currencies::free_balance(BTC, &LoansModule::account_id()), 200);
		assert_ok!(LoansModule::adjust_position(&ALICE, BTC, SETUSD, -100, -100));
		assert_eq!(LoansModule::setdollar_positions(BTC, &ALICE).debit, 100);
	});
}

#[test]
fn check_update_loan_underflow_work() {
	ExtBuilder::default().build().execute_with(|| {
		// collateral underflow
		assert_noop!(
			LoansModule::update_loan(&ALICE, BTC, SETUSD, -100, 0),
			Error::<Runtime>::CollateralTooLow,
		);

		// debit underflow
		assert_noop!(
			LoansModule::update_loan(&ALICE, BTC, SETUSD, 0, -100),
			Error::<Runtime>::DebitTooLow,
		);
	});
}

#[test]
fn adjust_position_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);

		// balance too low
		assert_noop!(
			LoansModule::adjust_position(&ALICE, BTC, SETR, 2000, 0),
			orml_tokens::Error::<Runtime>::BalanceTooLow
		);

		// mock can't pass position valid check
		assert_noop!(
			LoansModule::adjust_position(&ALICE, DNAR, SETR, 500, 0),
			sp_runtime::DispatchError::Other("mock invalid position error")
		);

		// mock exceed debit value cap
		assert_noop!(
			LoansModule::adjust_position(&ALICE, BTC, SETR, 1000, 1000),
			sp_runtime::DispatchError::Other("mock exceed debit value cap error")
		);

		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(BTC, &LoansModule::account_id()), 0);
		assert_eq!(LoansModule::total_setter_positions(BTC).debit, 0);
		assert_eq!(LoansModule::total_setter_positions(BTC).collateral, 0);
		assert_eq!(LoansModule::setter_positions(BTC, &ALICE).debit, 0);
		assert_eq!(LoansModule::setter_positions(BTC, &ALICE).collateral, 0);
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 0);

		// success
		assert_ok!(LoansModule::adjust_position(&ALICE, BTC, SETR, 500, 300));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 500);
		assert_eq!(Currencies::free_balance(BTC, &LoansModule::account_id()), 500);
		assert_eq!(LoansModule::total_setter_positions(BTC).debit, 300);
		assert_eq!(LoansModule::total_setter_positions(BTC).collateral, 500);
		assert_eq!(LoansModule::setter_positions(BTC, &ALICE).debit, 300);
		assert_eq!(LoansModule::setter_positions(BTC, &ALICE).collateral, 500);
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 150);
		Event::loans(crate::Event::PositionUpdated(ALICE, BTC, SETR, 500, 300));

		// collateral_adjustment is negatives
		// remove module account.
		assert_eq!(Currencies::total_balance(BTC, &LoansModule::account_id()), 500);
		assert_eq!(System::account_exists(&LoansModule::account_id()), true);
		assert_ok!(LoansModule::adjust_position(&ALICE, BTC, SETR, -500, 0));
		assert_eq!(Currencies::free_balance(BTC, &LoansModule::account_id()), 0);
		assert_eq!(System::account_exists(&LoansModule::account_id()), false);
	});
}

#[test]
fn update_loan_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(BTC, &LoansModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);
		assert_eq!(LoansModule::total_setdollar_positions(BTC).debit, 0);
		assert_eq!(LoansModule::total_setdollar_positions(BTC).collateral, 0);
		assert_eq!(LoansModule::setdollar_positions(BTC, &ALICE).debit, 0);
		assert_eq!(LoansModule::setdollar_positions(BTC, &ALICE).collateral, 0);
		assert_eq!(<SetDollarPositions<Runtime>>::contains_key(BTC, &ALICE), false);

		let alice_ref_count_0 = System::consumers(&ALICE);

		assert_ok!(LoansModule::update_loan(&ALICE, BTC, SETUSD, 3000, 2000));

		// just update records
		assert_eq!(LoansModule::total_setdollar_positions(BTC).debit, 2000);
		assert_eq!(LoansModule::total_setdollar_positions(BTC).collateral, 3000);
		assert_eq!(LoansModule::setdollar_positions(BTC, &ALICE).debit, 2000);
		assert_eq!(LoansModule::setdollar_positions(BTC, &ALICE).collateral, 3000);

		// increase ref count when open new position
		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 + 1);

		// dot not manipulate balance
		assert_eq!(Currencies::free_balance(BTC, &LoansModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);

		// should remove position storage if zero
		assert_eq!(<SetDollarPositions<Runtime>>::contains_key(BTC, SETUSD, &ALICE), true);
		assert_ok!(LoansModule::update_loan(&ALICE, BTC, SETUSD, -3000, -2000));
		assert_eq!(LoansModule::setdollar_positions(BTC, &ALICE).debit, 0);
		assert_eq!(LoansModule::setdollar_positions(BTC, &ALICE).collateral, 0);
		assert_eq!(<SetDollarPositions<Runtime>>::contains_key(BTC, &ALICE), false);

		// decrease ref count after remove position
		let alice_ref_count_2 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_2, alice_ref_count_1 - 1);
	});
}

#[test]
fn transfer_loan_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(LoansModule::update_loan(&ALICE, BTC, SETEUR, 400, 500));
		assert_ok!(LoansModule::update_loan(&BOB, BTC, SETEUR, 100, 600));
		assert_eq!(LoansModule::seteuro_positions(BTC, &ALICE).debit, 500);
		assert_eq!(LoansModule::seteuro_positions(BTC, &ALICE).collateral, 400);
		assert_eq!(LoansModule::seteuro_positions(BTC, &BOB).debit, 600);
		assert_eq!(LoansModule::seteuro_positions(BTC, &BOB).collateral, 100);

		assert_ok!(LoansModule::transfer_loan(&ALICE, &BOB, BTC, SETEUR));
		assert_eq!(LoansModule::seteuro_positions(BTC, &ALICE).debit, 0);
		assert_eq!(LoansModule::seteuro_positions(BTC, &ALICE).collateral, 0);
		assert_eq!(LoansModule::seteuro_positions(BTC, &BOB).debit, 1100);
		assert_eq!(LoansModule::seteuro_positions(BTC, &BOB).collateral, 500);
		Event::loans(crate::Event::TransferLoan(ALICE, BOB, BTC, SETEUR));
	});
}

#[test]
fn confiscate_collateral_and_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(LoansModule::update_loan(&BOB, BTC, SETGBP, 5000, 1000));
		assert_eq!(Currencies::free_balance(BTC, &LoansModule::account_id()), 0);

		// have no sufficient balance
		assert_eq!(
			LoansModule::confiscate_collateral_and_debit(&BOB, BTC, SETGBP, 5000, 1000).is_ok(),
			false,
		);

		assert_ok!(LoansModule::adjust_position(&ALICE, BTC, SETGBP, 500, 300));
		assert_eq!(CDPTreasuryModule::get_total_collaterals(BTC), 0);
		assert_eq!(CDPTreasuryModule::debit_pool(SETGBP), 0);
		assert_eq!(LoansModule::setpound_positions(BTC, &ALICE).debit, 300);
		assert_eq!(LoansModule::setpound_positions(BTC, &ALICE).collateral, 500);

		assert_ok!(LoansModule::confiscate_collateral_and_debit(&ALICE, BTC, SETGBP, 300, 200));
		assert_eq!(CDPTreasuryModule::get_total_collaterals(BTC), 300);
		assert_eq!(CDPTreasuryModule::debit_pool(SETGBP), 100);
		assert_eq!(LoansModule::setpound_positions(BTC, &ALICE).debit, 100);
		assert_eq!(LoansModule::setpound_positions(BTC, &ALICE).collateral, 200);
		Event::loans(crate::Event::ConfiscateCollateralAndDebit(
			ALICE, BTC, SETGBP, 300, 200,
		));
	});
}
