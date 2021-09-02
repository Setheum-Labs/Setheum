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

//! Unit tests for the SetmintGateway module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

#[test]
fn authorize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SetmintGateway::authorize(Origin::signed(ALICE), SETUSD, BOB));

		System::assert_last_event(Event::SetmintGateway(crate::Event::Authorization(ALICE, BOB, SETUSD)));

		assert_ok!(SetmintGateway::check_authorization(&ALICE, &BOB, SETUSD));
	});
}

#[test]
fn unauthorize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SetmintGateway::authorize(Origin::signed(ALICE), SETUSD, BOB));
		assert_ok!(SetmintGateway::check_authorization(&ALICE, &BOB, SETUSD));
		assert_ok!(SetmintGateway::unauthorize(Origin::signed(ALICE), SETUSD, BOB));

		System::assert_last_event(Event::SetmintGateway(crate::Event::UnAuthorization(ALICE, BOB, SETUSD)));

		assert_noop!(
			SetmintGateway::check_authorization(&ALICE, &BOB, SETUSD),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn unauthorize_all_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SetmintGateway::authorize(Origin::signed(ALICE), SETUSD, BOB));
		assert_ok!(SetmintGateway::authorize(Origin::signed(ALICE), SETUSD, CAROL));
		assert_ok!(SetmintGateway::unauthorize_all(Origin::signed(ALICE)));

		System::assert_last_event(Event::SetmintGateway(crate::Event::UnAuthorizationAll(ALICE)));

		assert_noop!(
			SetmintGateway::check_authorization(&ALICE, &BOB, SETUSD),
			Error::<Runtime>::NoPermission
		);
		assert_noop!(
			SetmintGateway::check_authorization(&ALICE, &BOB, SETUSD),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn transfer_position_from_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetmintGateway::adjust_position(Origin::signed(ALICE), SETUSD, 100, 50));
		assert_ok!(SetmintGateway::authorize(Origin::signed(ALICE), SETUSD, BOB));
		assert_ok!(SetmintGateway::transfer_position_from(Origin::signed(BOB), SETUSD, ALICE));
		assert_eq!(SetmintManagerModule::positions(SETUSD, BOB).reserve, 100);
		assert_eq!(SetmintManagerModule::positions(SETUSD, BOB).standard, 50);
	});
}

#[test]
fn transfer_unauthorization_setters_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			SetmintGateway::transfer_position_from(Origin::signed(ALICE), SETUSD, BOB),
			Error::<Runtime>::NoPermission,
		);
	});
}

#[test]
fn adjust_position_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetmintGateway::adjust_position(Origin::signed(ALICE), SETUSD, 100, 50));
		assert_eq!(SetmintManagerModule::positions(SETUSD, ALICE).reserve, 100);
		assert_eq!(SetmintManagerModule::positions(SETUSD, ALICE).standard, 50);
	});
}
