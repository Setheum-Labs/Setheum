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

//! Unit tests for the settmint engine module.

#![cfg(test)]

use super::*;
use frame_support::assert_ok;
use mock::*;
use orml_traits::MultiCurrency;

#[test]
fn get_standard_exchange_rate_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SettmintEngineModule::get_standard_exchange_rate(SETT),
			DefaultStandardExchangeRate::get()
		);
	});
}

#[test]
fn calculate_reserve_ratio_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SettmintEngineModule::calculate_reserve_ratio(SETT, 100, 50, Price::saturating_from_rational(1, 1)),
			Ratio::saturating_from_rational(100, 50)
		);
	});
}

#[test]
fn adjust_position_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 0);
		assert_eq!(SettmintManagerModule::positions(USDJ, ALICE).standard, 0);
		assert_eq!(SettmintManagerModule::positions(USDJ, ALICE).reserve, 0);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, USDJ, 100, 50));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 50);
		assert_eq!(SettmintManagerModule::positions(USDJ, ALICE).standard, 50);
		assert_eq!(SettmintManagerModule::positions(USDJ, ALICE).reserve, 100);
		assert_eq!(SettmintEngineModule::adjust_position(&ALICE, USDJ, 0, 20).is_ok(), true);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, USDJ, 0, -20));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 50);
		assert_eq!(SettmintManagerModule::positions(USDJ, ALICE).standard, 50);
		assert_eq!(SettmintManagerModule::positions(USDJ, ALICE).reserve, 100);
	});
}
