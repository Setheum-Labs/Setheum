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

//! Unit tests for the setters-manager module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

#[test]
fn standards_key() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).standard, 0);
		assert_ok!(SettersManagerModule::adjust_position(&ALICE, EURJ, 100, 100));
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).standard, 100);
		assert_ok!(SettersManagerModule::adjust_position(&ALICE, EURJ, -100, -100));
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).standard, 0);
	});
}

#[test]
fn check_update_reserve_overflow_work() {
	ExtBuilder::default().build().execute_with(|| {
		// reserve underflow
		assert_noop!(
			SettersManagerModule::update_reserve(&ALICE, EURJ, -100, 0),
			Error::<Runtime>::ReserveTooLow,
		);

		// standard underflow
		assert_noop!(
			SettersManagerModule::update_reserve(&ALICE, EURJ, 0, -100),
			Error::<Runtime>::StandardTooLow,
		);
	});
}

#[test]
fn adjust_position_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 1000);

		// balance too low
		assert_eq!(SettersManagerModule::adjust_position(&ALICE, EURJ, 2000, 0).is_ok(), false);

		// mock can't pass position valid check
		assert_eq!(SettersManagerModule::adjust_position(&ALICE, USDJ, 500, 0).is_ok(), false);

		assert_eq!(Currencies::free_balance(SETT, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(EURJ, &SettersManagerModule::account_id()), 0);
		assert_eq!(SettersManagerModule::total_positions(EURJ).standard, 0);
		assert_eq!(SettersManagerModule::total_positions(EURJ).reserve, 0);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).standard, 0);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).reserve, 0);
		assert_eq!(Currencies::free_balance(EURJ, &ALICE), 0);

		// success
		assert_ok!(SettersManagerModule::adjust_position(&ALICE, EURJ, 500, 300));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 500);
		assert_eq!(Currencies::free_balance(SETT, &SettersManagerModule::account_id()), 500);
		assert_eq!(SettersManagerModule::total_positions(EURJ).standard, 300);
		assert_eq!(SettersManagerModule::total_positions(EURJ).reserve, 500);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).standard, 300);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).reserve, 500);
		assert_eq!(Currencies::free_balance(EURJ, &ALICE), 150);

		System::assert_last_event(Event::setters_manager(crate::Event::PositionUpdated(ALICE, EURJ, 500, 300)));
	});
}

#[test]
fn transfer_reserve_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SettersManagerModule::update_reserve(&ALICE, EURJ, 400, 500));
		assert_ok!(SettersManagerModule::update_reserve(&BOB, EURJ, 100, 600));
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).standard, 500);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).reserve, 400);
		assert_eq!(SettersManagerModule::positions(EURJ, &BOB).standard, 600);
		assert_eq!(SettersManagerModule::positions(EURJ, &BOB).reserve, 100);

		assert_ok!(SettersManagerModule::transfer_reserve(&ALICE, &BOB, EURJ));
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).standard, 0);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).reserve, 0);
		assert_eq!(SettersManagerModule::positions(EURJ, &BOB).standard, 1100);
		assert_eq!(SettersManagerModule::positions(EURJ, &BOB).reserve, 500);

		System::assert_last_event(Event::setters_manager(crate::Event::TransferReserve(ALICE, BOB, EURJ)));
	});
}

#[test]
fn update_reserve_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETT, &SettersManagerModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 1000);
		assert_eq!(SettersManagerModule::total_positions(EURJ).standard, 0);
		assert_eq!(SettersManagerModule::total_positions(EURJ).reserve, 0);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).standard, 0);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).reserve, 0);
		assert_eq!(<Positions<Runtime>>::contains_key(EURJ, &ALICE), false);

		let alice_ref_count_0 = System::consumers(&ALICE);

		assert_ok!(SettersManagerModule::update_reserve(&ALICE, EURJ, 3000, 2000));

		// just update records
		assert_eq!(SettersManagerModule::total_positions(EURJ).standard, 2000);
		assert_eq!(SettersManagerModule::total_positions(EURJ).reserve, 3000);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).standard, 2000);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).reserve, 3000);

		// increase ref count when open new position
		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 + 1);

		// dot not manipulate balance
		assert_eq!(Currencies::free_balance(SETT, &SettersManagerModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 1000);

		// should remove position storage if zero
		assert_eq!(<Positions<Runtime>>::contains_key(EURJ, &ALICE), true);
		assert_ok!(SettersManagerModule::update_reserve(&ALICE, EURJ, -3000, -2000));
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).standard, 0);
		assert_eq!(SettersManagerModule::positions(EURJ, &ALICE).reserve, 0);
		assert_eq!(<Positions<Runtime>>::contains_key(EURJ, &ALICE), false);

		// decrease ref count after remove position
		let alice_ref_count_2 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_2, alice_ref_count_1 - 1);
	});
}

#[test]
fn total_reserve_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SettersManagerModule::total_reserve(0);
		assert_ok!(Currencies::deposit(SETT,&SettersManagerModule::account_id()), 10));
		assert_eq!(SettersManagerModule::total_reserve(10);
	});
}

#[test]
fn get_total_reserve_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettersManagerModule::deposit_setter(&ALICE, 500));
		assert_eq!(SettersManagerModule::get_total_reserve(500);
	});
}
