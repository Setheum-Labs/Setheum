// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
//
// This file is part of Zims.
//
// Copyright (C) 2019-2022 Setheum Labs.
// SPDX-License-Identifier: BUSL-1.1 (Business Source License 1.1)

//! Unit tests for the emergency shutdown module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn emergency_shutdown_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert!(!EmergencyShutdownModule::is_shutdown());
		assert_noop!(
			EmergencyShutdownModule::emergency_shutdown(Origin::signed(5)),
			BadOrigin,
		);
		assert_ok!(EmergencyShutdownModule::emergency_shutdown(Origin::signed(1)));
		System::assert_last_event(Event::EmergencyShutdownModule(crate::Event::Shutdown {
			block_number: 1,
		}));
		assert!(EmergencyShutdownModule::is_shutdown());
		assert_noop!(
			EmergencyShutdownModule::emergency_shutdown(Origin::signed(1)),
			Error::<Runtime>::AlreadyShutdown,
		);
	});
}

#[test]
fn open_collateral_refund_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(!EmergencyShutdownModule::can_refund());
		assert_noop!(
			EmergencyShutdownModule::open_collateral_refund(Origin::signed(1)),
			Error::<Runtime>::MustAfterShutdown,
		);
	});
}

#[test]
fn open_collateral_refund_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert!(!EmergencyShutdownModule::can_refund());
		assert_ok!(EmergencyShutdownModule::emergency_shutdown(Origin::signed(1)));
		assert_noop!(
			EmergencyShutdownModule::open_collateral_refund(Origin::signed(5)),
			BadOrigin,
		);
		assert_ok!(EmergencyShutdownModule::open_collateral_refund(Origin::signed(1)));
		System::assert_last_event(Event::EmergencyShutdownModule(crate::Event::OpenRefund {
			block_number: 1,
		}));
		assert!(EmergencyShutdownModule::can_refund());
	});
}

#[test]
fn refund_collaterals_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EmergencyShutdownModule::refund_collaterals(Origin::signed(ALICE), 10),
			Error::<Runtime>::CanNotRefund,
		);
	});
}
