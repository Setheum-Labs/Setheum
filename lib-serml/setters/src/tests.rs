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

//! Unit tests for the setters module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

#[test]
fn standards_key() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SettersModule::positions(BTC, &ALICE).standard, 0);
		assert_ok!(SettersModule::adjust_position(&ALICE, BTC, 100, 100));
		assert_eq!(SettersModule::positions(BTC, &ALICE).standard, 100);
		assert_ok!(SettersModule::adjust_position(&ALICE, BTC, -100, -100));
		assert_eq!(SettersModule::positions(BTC, &ALICE).standard, 0);
	});
}

#[test]
fn check_update_setter_overflow_work() {
	ExtBuilder::default().build().execute_with(|| {
		// reserve underflow
		assert_noop!(
			SettersModule::update_setter(&ALICE, BTC, -100, 0),
			Error::<Runtime>::ReserveTooLow,
		);

		// standard underflow
		assert_noop!(
			SettersModule::update_setter(&ALICE, BTC, 0, -100),
			Error::<Runtime>::StandardTooLow,
		);
	});
}

#[test]
fn adjust_position_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);

		// balance too low
		assert_eq!(SettersModule::adjust_position(&ALICE, BTC, 2000, 0).is_ok(), false);

		// mock can't pass position valid check
		assert_eq!(SettersModule::adjust_position(&ALICE, DOT, 500, 0).is_ok(), false);

		// mock exceed standard value cap
		assert_eq!(SettersModule::adjust_position(&ALICE, BTC, 1000, 1000).is_ok(), false);

		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(BTC, &SettersModule::account_id()), 0);
		assert_eq!(SettersModule::total_positions(BTC).standard, 0);
		assert_eq!(SettersModule::total_positions(BTC).reserve, 0);
		assert_eq!(SettersModule::positions(BTC, &ALICE).standard, 0);
		assert_eq!(SettersModule::positions(BTC, &ALICE).reserve, 0);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 0);

		// success
		assert_ok!(SettersModule::adjust_position(&ALICE, BTC, 500, 300));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 500);
		assert_eq!(Currencies::free_balance(BTC, &SettersModule::account_id()), 500);
		assert_eq!(SettersModule::total_positions(BTC).standard, 300);
		assert_eq!(SettersModule::total_positions(BTC).reserve, 500);
		assert_eq!(SettersModule::positions(BTC, &ALICE).standard, 300);
		assert_eq!(SettersModule::positions(BTC, &ALICE).reserve, 500);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 150);

		System::assert_last_event(Event::setters(crate::Event::PositionUpdated(ALICE, BTC, 500, 300)));
	});
}

#[test]
fn update_setter_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(BTC, &SettersModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);
		assert_eq!(SettersModule::total_positions(BTC).standard, 0);
		assert_eq!(SettersModule::total_positions(BTC).reserve, 0);
		assert_eq!(SettersModule::positions(BTC, &ALICE).standard, 0);
		assert_eq!(SettersModule::positions(BTC, &ALICE).reserve, 0);
		assert_eq!(<Positions<Runtime>>::contains_key(BTC, &ALICE), false);

		let alice_ref_count_0 = System::consumers(&ALICE);

		assert_ok!(SettersModule::update_setter(&ALICE, BTC, 3000, 2000));

		// just update records
		assert_eq!(SettersModule::total_positions(BTC).standard, 2000);
		assert_eq!(SettersModule::total_positions(BTC).reserve, 3000);
		assert_eq!(SettersModule::positions(BTC, &ALICE).standard, 2000);
		assert_eq!(SettersModule::positions(BTC, &ALICE).reserve, 3000);

		// increase ref count when open new position
		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 + 1);

		// dot not manipulate balance
		assert_eq!(Currencies::free_balance(BTC, &SettersModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);

		// should remove position storage if zero
		assert_eq!(<Positions<Runtime>>::contains_key(BTC, &ALICE), true);
		assert_ok!(SettersModule::update_setter(&ALICE, BTC, -3000, -2000));
		assert_eq!(SettersModule::positions(BTC, &ALICE).standard, 0);
		assert_eq!(SettersModule::positions(BTC, &ALICE).reserve, 0);
		assert_eq!(<Positions<Runtime>>::contains_key(BTC, &ALICE), false);

		// decrease ref count after remove position
		let alice_ref_count_2 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_2, alice_ref_count_1 - 1);
	});
}

#[test]
fn transfer_setter_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SettersModule::update_setter(&ALICE, BTC, 400, 500));
		assert_ok!(SettersModule::update_setter(&BOB, BTC, 100, 600));
		assert_eq!(SettersModule::positions(BTC, &ALICE).standard, 500);
		assert_eq!(SettersModule::positions(BTC, &ALICE).reserve, 400);
		assert_eq!(SettersModule::positions(BTC, &BOB).standard, 600);
		assert_eq!(SettersModule::positions(BTC, &BOB).reserve, 100);

		assert_ok!(SettersModule::transfer_setter(&ALICE, &BOB, BTC));
		assert_eq!(SettersModule::positions(BTC, &ALICE).standard, 0);
		assert_eq!(SettersModule::positions(BTC, &ALICE).reserve, 0);
		assert_eq!(SettersModule::positions(BTC, &BOB).standard, 1100);
		assert_eq!(SettersModule::positions(BTC, &BOB).reserve, 500);

		System::assert_last_event(Event::setters(crate::Event::TransferSetter(ALICE, BOB, BTC)));
	});
}
