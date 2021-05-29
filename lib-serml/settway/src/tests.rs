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
		assert_ok!(SettwayModule::authorize(Origin::signed(ALICE), BTC, BOB));

		let authorization_event = Event::settway(crate::Event::Authorization(ALICE, BOB, BTC));
		assert!(System::events()
			.iter()
			.any(|record| record.event == authorization_event));

		assert_ok!(SettwayModule::check_authorization(&ALICE, &BOB, BTC));
	});
}

#[test]
fn unauthorize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SettwayModule::authorize(Origin::signed(ALICE), BTC, BOB));
		assert_ok!(SettwayModule::check_authorization(&ALICE, &BOB, BTC));
		assert_ok!(SettwayModule::unauthorize(Origin::signed(ALICE), BTC, BOB));

		let unauthorization_event = Event::settway(crate::Event::UnAuthorization(ALICE, BOB, BTC));
		assert!(System::events()
			.iter()
			.any(|record| record.event == unauthorization_event));

		assert_noop!(
			SettwayModule::check_authorization(&ALICE, &BOB, BTC),
			Error::<Runtime>::NoAuthorization
		);
	});
}

#[test]
fn unauthorize_all_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SettwayModule::authorize(Origin::signed(ALICE), BTC, BOB));
		assert_ok!(SettwayModule::authorize(Origin::signed(ALICE), DOT, CAROL));
		assert_ok!(SettwayModule::unauthorize_all(Origin::signed(ALICE)));

		let unauthorization_all_event = Event::settway(crate::Event::UnAuthorizationAll(ALICE));
		assert!(System::events()
			.iter()
			.any(|record| record.event == unauthorization_all_event));

		assert_noop!(
			SettwayModule::check_authorization(&ALICE, &BOB, BTC),
			Error::<Runtime>::NoAuthorization
		);
		assert_noop!(
			SettwayModule::check_authorization(&ALICE, &BOB, DOT),
			Error::<Runtime>::NoAuthorization
		);
	});
}

#[test]
fn transfer_setter_from_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SettwayModule::adjust_setter(Origin::signed(ALICE), BTC, 100, 50));
		assert_ok!(SettwayModule::authorize(Origin::signed(ALICE), BTC, BOB));
		assert_ok!(SettwayModule::transfer_setter_from(Origin::signed(BOB), BTC, ALICE));
		assert_eq!(SettersModule::positions(BTC, BOB).reserve, 100);
		assert_eq!(SettersModule::positions(BTC, BOB).standard, 50);
	});
}

#[test]
fn transfer_unauthorization_setters_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			SettwayModule::transfer_setter_from(Origin::signed(ALICE), BTC, BOB),
			Error::<Runtime>::NoAuthorization,
		);
	});
}

#[test]
fn adjust_setter_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SettwayModule::adjust_setter(Origin::signed(ALICE), BTC, 100, 50));
		assert_eq!(SettersModule::positions(BTC, ALICE).reserve, 100);
		assert_eq!(SettersModule::positions(BTC, ALICE).standard, 50);
	});
}

#[test]
fn on_emergency_shutdown_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		mock_shutdown();
		assert_noop!(
			SettwayModule::adjust_setter(Origin::signed(ALICE), BTC, 100, 50),
			Error::<Runtime>::AlreadyShutdown,
		);
		assert_noop!(
			SettwayModule::transfer_setter_from(Origin::signed(ALICE), BTC, BOB),
			Error::<Runtime>::AlreadyShutdown,
		);
	});
}
