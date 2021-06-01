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

//! Unit tests for the settmint engine module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use orml_traits::MultiCurrency;
use sp_runtime::traits::BadOrigin;

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
fn set_global_params_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			SettmintEngineModule::set_global_params(Origin::signed(5), Rate::saturating_from_rational(1, 10000)),
			BadOrigin
		);
		assert_ok!(SettmintEngineModule::set_global_params(
			Origin::signed(1),
			Rate::saturating_from_rational(1, 10000),
		));

		let update_global_stability_fee_event = Event::settmint_engine(crate::Event::GlobalStabilityFeeUpdated(
			Rate::saturating_from_rational(1, 10000),
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_global_stability_fee_event));

		assert_eq!(
			SettmintEngineModule::global_stability_fee(),
			Rate::saturating_from_rational(1, 10000)
		);
	});
}

#[test]
fn set_reserve_params_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			SettmintEngineModule::set_reserve_params(
				Origin::signed(1),
				DNAR,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
			),
			Error::<Runtime>::InvalidReserveType
		);

		System::set_block_number(1);
		assert_noop!(
			SettmintEngineModule::set_reserve_params(
				Origin::signed(5),
				SETT,
				Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
				Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
				Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
				Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
				Change::NewValue(10000),
			),
			BadOrigin
		);
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		let update_stability_fee_event = Event::settmint_engine(crate::Event::StabilityFeeUpdated(
			SETT,
			Some(Rate::saturating_from_rational(1, 100000)),
		));
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		let new_reserve_params = SettmintEngineModule::reserve_params(SETT);

		assert_eq!(
			new_reserve_params.stability_fee,
			Some(Rate::saturating_from_rational(1, 100000))
		);
		assert_eq!(
			new_reserve_params.required_reserve_ratio,
			Some(Ratio::saturating_from_rational(9, 5))
		);
	});
}

#[test]
fn calculate_reserve_ratio_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			SettmintEngineModule::calculate_reserve_ratio(SETT, 100, 50, Price::saturating_from_rational(1, 1)),
			Ratio::saturating_from_rational(100, 50)
		);
	});
}

#[test]
fn check_standard_cap_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SettmintEngineModule::check_standard_cap(SETT, 9999));
		assert_noop!(
			SettmintEngineModule::check_standard_cap(SETT, 10001),
			Error::<Runtime>::ExceedStandardValueHardCap,
		);
	});
}

#[test]
fn check_position_valid_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(1, 1))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(10000),
		));

		MockPriceSource::set_relative_price(None);
		assert_noop!(
			SettmintEngineModule::check_position_valid(SETT, 100, 50),
			Error::<Runtime>::InvalidFeedPrice
		);
		MockPriceSource::set_relative_price(Some(Price::one()));

		assert_ok!(SettmintEngineModule::check_position_valid(SETT, 100, 50));
	});
}

#[test]
fn check_position_valid_failed_when_remain_standard_value_too_small() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(1, 1))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(10000),
		));
		assert_noop!(
			SettmintEngineModule::check_position_valid(SETT, 2, 1),
			Error::<Runtime>::RemainStandardValueTooSmall,
		);
	});
}

#[test]
fn check_position_valid_ratio_below_required_ratio() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			SettmintEngineModule::check_position_valid(SETT, 89, 50),
			Error::<Runtime>::BelowRequiredReserveRatio
		);
	});
}

#[test]
fn adjust_position_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			SettmintEngineModule::adjust_position(&ALICE, DNAR, 100, 50),
			Error::<Runtime>::InvalidReserveType,
		);
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 0);
		assert_eq!(SettersModule::positions(SETT, ALICE).standard, 0);
		assert_eq!(SettersModule::positions(SETT, ALICE).reserve, 0);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, SETT, 100, 50));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 50);
		assert_eq!(SettersModule::positions(SETT, ALICE).standard, 50);
		assert_eq!(SettersModule::positions(SETT, ALICE).reserve, 100);
		assert_eq!(SettmintEngineModule::adjust_position(&ALICE, SETT, 0, 20).is_ok(), false);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, SETT, 0, -20));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 30);
		assert_eq!(SettersModule::positions(SETT, ALICE).standard, 30);
		assert_eq!(SettersModule::positions(SETT, ALICE).reserve, 100);
	});
}

#[test]
fn remain_standard_value_too_small_check() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, SETT, 100, 50));
		assert_eq!(SettmintEngineModule::adjust_position(&ALICE, SETT, 0, -49).is_ok(), false);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, SETT, -100, -50));
	});
}

#[test]
fn on_finalize_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			SETT,
			Change::NewValue(Some(Rate::saturating_from_rational(2, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		SettmintEngineModule::on_finalize(1);
		assert_eq!(SettmintEngineModule::standard_exchange_rate(SETT), None);
		assert_eq!(SettmintEngineModule::standard_exchange_rate(SETT), None);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, SETT, 100, 30));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 30);
		SettmintEngineModule::on_finalize(2);
		assert_eq!(
			SettmintEngineModule::standard_exchange_rate(SETT),
			Some(ExchangeRate::saturating_from_rational(101, 100))
		);
		assert_eq!(SettmintEngineModule::standard_exchange_rate(SETT), None);
		SettmintEngineModule::on_finalize(3);
		assert_eq!(
			SettmintEngineModule::standard_exchange_rate(SETT),
			Some(ExchangeRate::saturating_from_rational(10201, 10000))
		);
		assert_eq!(SettmintEngineModule::standard_exchange_rate(SETT), None);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, SETT, 0, -30));
		assert_eq!(Currencies::free_balance(SETT, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 0);
		SettmintEngineModule::on_finalize(4);
		assert_eq!(
			SettmintEngineModule::standard_exchange_rate(SETT),
			Some(ExchangeRate::saturating_from_rational(10201, 10000))
		);
		assert_eq!(SettmintEngineModule::standard_exchange_rate(SETT), None);
	});
}
