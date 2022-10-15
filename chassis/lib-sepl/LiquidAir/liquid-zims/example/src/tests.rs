// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
//
// This file is part of LiquidAir.
//
// Copyright (C) 2019-2022 Setheum Labs.
// SPDX-License-Identifier: BUSL-1.1 (Business Source License 1.1)

//! Unit tests for example module.

#![cfg(test)]

use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn set_dummy_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Example::dummy(), None);
		assert_ok!(Example::set_dummy(Origin::root(), 20));
		assert_eq!(Example::dummy(), Some(20));
		System::assert_last_event(Event::Example(crate::Event::Dummy(20)));
	});
}

#[test]
fn do_set_bar_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Example::bar(2), 200);
		Example::do_set_bar(&2, 10);
		assert_eq!(Example::bar(2), 10);
	});
}
