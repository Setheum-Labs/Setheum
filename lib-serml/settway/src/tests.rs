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

//! Unit tests for the settway module.

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
		assert_ok!(SettwayModule::authorize(Origin::signed(ALICE), SETT, BOB));

		System::assert_last_event(Event::settway(crate::Event::Authorization(ALICE, BOB, SETT)));

		assert_ok!(SettwayModule::check_authorization(&ALICE, &BOB, SETT));
	});
}

#[test]
fn unauthorize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SettwayModule::authorize(Origin::signed(ALICE), SETT, BOB));
		assert_ok!(SettwayModule::check_authorization(&ALICE, &BOB, SETT));
		assert_ok!(SettwayModule::unauthorize(Origin::signed(ALICE), SETT, BOB));

		System::assert_last_event(Event::settway(crate::Event::UnAuthorization(ALICE, BOB, SETT)));

		assert_noop!(
			SettwayModule::check_authorization(&ALICE, &BOB, SETT),
			Error::<Runtime>::NoAuthorization
		);
	});
}

#[test]
fn unauthorize_all_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SettwayModule::authorize(Origin::signed(ALICE), SETT, BOB));
		assert_ok!(SettwayModule::authorize(Origin::signed(ALICE), SETT, CAROL));
		assert_ok!(SettwayModule::unauthorize_all(Origin::signed(ALICE)));

		System::assert_last_event(Event::settway(crate::Event::UnAuthorizationAll(ALICE)));

		assert_noop!(
			SettwayModule::check_authorization(&ALICE, &BOB, SETT),
			Error::<Runtime>::NoAuthorization
		);
		assert_noop!(
			SettwayModule::check_authorization(&ALICE, &BOB, SETT),
			Error::<Runtime>::NoAuthorization
		);
	});
}

#[test]
fn transfer_setter_from_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettwayModule::adjust_setter(Origin::signed(ALICE), SETT, 100, 50));
		assert_ok!(SettwayModule::authorize(Origin::signed(ALICE), SETT, BOB));
		assert_ok!(SettwayModule::transfer_setter_from(Origin::signed(BOB), SETT, ALICE));
		assert_eq!(SettersModule::positions(SETT, BOB).reserve, 100);
		assert_eq!(SettersModule::positions(SETT, BOB).standard, 50);
	});
}

#[test]
fn transfer_unauthorization_setters_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			SettwayModule::transfer_setter_from(Origin::signed(ALICE), SETT, BOB),
			Error::<Runtime>::NoAuthorization,
		);
	});
}

#[test]
fn adjust_setter_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettwayModule::adjust_setter(Origin::signed(ALICE), SETT, 100, 50));
		assert_eq!(SettersModule::positions(SETT, ALICE).reserve, 100);
		assert_eq!(SettersModule::positions(SETT, ALICE).standard, 50);
	});
}
