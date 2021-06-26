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

//! Unit tests for the settmint_gateway module.

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
		assert_ok!(SettmintGatewayModule::authorize(Origin::signed(ALICE), USDJ, BOB));

		System::assert_last_event(Event::settmint_gateway(crate::Event::Authorization(ALICE, BOB, USDJ)));

		assert_ok!(SettmintGatewayModule::check_authorization(&ALICE, &BOB, USDJ));
	});
}

#[test]
fn unauthorize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SettmintGatewayModule::authorize(Origin::signed(ALICE), USDJ, BOB));
		assert_ok!(SettmintGatewayModule::check_authorization(&ALICE, &BOB, USDJ));
		assert_ok!(SettmintGatewayModule::unauthorize(Origin::signed(ALICE), USDJ, BOB));

		System::assert_last_event(Event::settmint_gateway(crate::Event::UnAuthorization(ALICE, BOB, USDJ)));

		assert_noop!(
			SettmintGatewayModule::check_authorization(&ALICE, &BOB, USDJ),
			Error::<Runtime>::NoAuthorization
		);
	});
}

#[test]
fn unauthorize_all_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SettmintGatewayModule::authorize(Origin::signed(ALICE), USDJ, BOB));
		assert_ok!(SettmintGatewayModule::authorize(Origin::signed(ALICE), USDJ, CAROL));
		assert_ok!(SettmintGatewayModule::unauthorize_all(Origin::signed(ALICE)));

		System::assert_last_event(Event::settmint_gateway(crate::Event::UnAuthorizationAll(ALICE)));

		assert_noop!(
			SettmintGatewayModule::check_authorization(&ALICE, &BOB, USDJ),
			Error::<Runtime>::NoAuthorization
		);
		assert_noop!(
			SettmintGatewayModule::check_authorization(&ALICE, &BOB, USDJ),
			Error::<Runtime>::NoAuthorization
		);
	});
}

#[test]
fn transfer_position_from_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintGatewayModule::adjust_position(Origin::signed(ALICE), USDJ, 100, 50));
		assert_ok!(SettmintGatewayModule::authorize(Origin::signed(ALICE), USDJ, BOB));
		assert_ok!(SettmintGatewayModule::transfer_position_from(Origin::signed(BOB), USDJ, ALICE));
		assert_eq!(SettmintManagerModule::positions(USDJ, BOB).reserve, 100);
		assert_eq!(SettmintManagerModule::positions(USDJ, BOB).standard, 50);
	});
}

#[test]
fn transfer_unauthorization_setters_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			SettmintGatewayModule::transfer_position_from(Origin::signed(ALICE), USDJ, BOB),
			Error::<Runtime>::NoAuthorization,
		);
	});
}

#[test]
fn adjust_position_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintGatewayModule::adjust_position(Origin::signed(ALICE), USDJ, 100, 50));
		assert_eq!(SettmintManagerModule::positions(USDJ, ALICE).reserve, 100);
		assert_eq!(SettmintManagerModule::positions(USDJ, ALICE).standard, 50);
	});
}
