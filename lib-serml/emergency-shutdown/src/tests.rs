// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
// ٱلَّذِينَ يَأْكُلُونَ ٱلرِّبَوٰا۟ لَا يَقُومُونَ إِلَّا كَمَا يَقُومُ ٱلَّذِى يَتَخَبَّطُهُ ٱلشَّيْطَـٰنُ مِنَ ٱلْمَسِّ ۚ ذَٰلِكَ بِأَنَّهُمْ قَالُوٓا۟ إِنَّمَا ٱلْبَيْعُ مِثْلُ ٱلرِّبَوٰا۟ ۗ وَأَحَلَّ ٱللَّهُ ٱلْبَيْعَ وَحَرَّمَ ٱلرِّبَوٰا۟ ۚ فَمَن جَآءَهُۥ مَوْعِظَةٌ مِّن رَّبِّهِۦ فَٱنتَهَىٰ فَلَهُۥ مَا سَلَفَ وَأَمْرُهُۥٓ إِلَى ٱللَّهِ ۖ وَمَنْ عَادَ فَأُو۟لَـٰٓئِكَ أَصْحَـٰبُ ٱلنَّارِ ۖ هُمْ فِيهَا خَـٰلِدُونَ

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
		System::assert_last_event(Event::EmergencyShutdownModule(crate::Event::Shutdown(1)));
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
		System::assert_last_event(Event::EmergencyShutdownModule(crate::Event::OpenRefund(1)));
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
