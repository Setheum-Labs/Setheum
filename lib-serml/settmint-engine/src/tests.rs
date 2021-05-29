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
fn is_settmint_unsafe_work() {
	fn is_user_safe(currency_id: CurrencyId, who: &AccountId) -> bool {
		let Position { reserve, standard } = SettersModule::positions(currency_id, &who);
		SettmintEngineModule::is_settmint_unsafe(currency_id, reserve, standard)
	}

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
		assert_eq!(is_user_safe(BTC, &ALICE), false);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, 100, 50));
		assert_eq!(is_user_safe(BTC, &ALICE), false);
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_eq!(is_user_safe(BTC, &ALICE), true);
	});
}

#[test]
fn get_standard_exchange_rate_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SettmintEngineModule::get_standard_exchange_rate(BTC),
			DefaultStandardExchangeRate::get()
		);
	});
}

#[test]
fn get_liquidation_penalty_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SettmintEngineModule::get_liquidation_penalty(BTC),
			DefaultLiquidationPenalty::get()
		);
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(5, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			SettmintEngineModule::get_liquidation_penalty(BTC),
			Rate::saturating_from_rational(2, 10)
		);
	});
}

#[test]
fn get_liquidation_ratio_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SettmintEngineModule::get_liquidation_ratio(BTC),
			DefaultLiquidationRatio::get()
		);
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(5, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			SettmintEngineModule::get_liquidation_ratio(BTC),
			Ratio::saturating_from_rational(5, 2)
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
				LDOT,
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
				BTC,
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
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		let update_stability_fee_event = Event::settmint_engine(crate::Event::StabilityFeeUpdated(
			BTC,
			Some(Rate::saturating_from_rational(1, 100000)),
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_stability_fee_event));
		let update_liquidation_ratio_event = Event::settmint_engine(crate::Event::LiquidationRatioUpdated(
			BTC,
			Some(Ratio::saturating_from_rational(3, 2)),
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_liquidation_ratio_event));
		let update_liquidation_penalty_event = Event::settmint_engine(crate::Event::LiquidationPenaltyUpdated(
			BTC,
			Some(Rate::saturating_from_rational(2, 10)),
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_liquidation_penalty_event));
		let update_required_reserve_ratio_event = Event::settmint_engine(crate::Event::RequiredReserveRatioUpdated(
			BTC,
			Some(Ratio::saturating_from_rational(9, 5)),
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_required_reserve_ratio_event));
		let update_maximum_total_standard_value_event =
			Event::settmint_engine(crate::Event::MaximumTotalStandardValueUpdated(BTC, 10000));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_maximum_total_standard_value_event));

		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		let new_reserve_params = SettmintEngineModule::reserve_params(BTC);

		assert_eq!(
			new_reserve_params.stability_fee,
			Some(Rate::saturating_from_rational(1, 100000))
		);
		assert_eq!(
			new_reserve_params.liquidation_ratio,
			Some(Ratio::saturating_from_rational(3, 2))
		);
		assert_eq!(
			new_reserve_params.liquidation_penalty,
			Some(Rate::saturating_from_rational(2, 10))
		);
		assert_eq!(
			new_reserve_params.required_reserve_ratio,
			Some(Ratio::saturating_from_rational(9, 5))
		);
		assert_eq!(new_reserve_params.maximum_total_standard_value, 10000);
	});
}

#[test]
fn calculate_reserve_ratio_work() {
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
		assert_eq!(
			SettmintEngineModule::calculate_reserve_ratio(BTC, 100, 50, Price::saturating_from_rational(1, 1)),
			Ratio::saturating_from_rational(100, 50)
		);
	});
}

#[test]
fn check_standard_cap_work() {
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
		assert_ok!(SettmintEngineModule::check_standard_cap(BTC, 9999));
		assert_noop!(
			SettmintEngineModule::check_standard_cap(BTC, 10001),
			Error::<Runtime>::ExceedStandardValueHardCap,
		);
	});
}

#[test]
fn check_position_valid_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(1, 1))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(10000),
		));

		MockPriceSource::set_relative_price(None);
		assert_noop!(
			SettmintEngineModule::check_position_valid(BTC, 100, 50),
			Error::<Runtime>::InvalidFeedPrice
		);
		MockPriceSource::set_relative_price(Some(Price::one()));

		assert_ok!(SettmintEngineModule::check_position_valid(BTC, 100, 50));
	});
}

#[test]
fn check_position_valid_failed_when_remain_standard_value_too_small() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(1, 1))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(10000),
		));
		assert_noop!(
			SettmintEngineModule::check_position_valid(BTC, 2, 1),
			Error::<Runtime>::RemainStandardValueTooSmall,
		);
	});
}

#[test]
fn check_position_valid_ratio_below_liquidate_ratio() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(10, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			SettmintEngineModule::check_position_valid(BTC, 91, 50),
			Error::<Runtime>::BelowLiquidationRatio,
		);
	});
}

#[test]
fn check_position_valid_ratio_below_required_ratio() {
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
		assert_noop!(
			SettmintEngineModule::check_position_valid(BTC, 89, 50),
			Error::<Runtime>::BelowRequiredReserveRatio
		);
	});
}

#[test]
fn adjust_position_work() {
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
		assert_noop!(
			SettmintEngineModule::adjust_position(&ALICE, DNAR, 100, 50),
			Error::<Runtime>::InvalidReserveType,
		);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 0);
		assert_eq!(SettersModule::positions(BTC, ALICE).standard, 0);
		assert_eq!(SettersModule::positions(BTC, ALICE).reserve, 0);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, 100, 50));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 50);
		assert_eq!(SettersModule::positions(BTC, ALICE).standard, 50);
		assert_eq!(SettersModule::positions(BTC, ALICE).reserve, 100);
		assert_eq!(SettmintEngineModule::adjust_position(&ALICE, BTC, 0, 20).is_ok(), false);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, 0, -20));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 30);
		assert_eq!(SettersModule::positions(BTC, ALICE).standard, 30);
		assert_eq!(SettersModule::positions(BTC, ALICE).reserve, 100);
	});
}

#[test]
fn remain_standard_value_too_small_check() {
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
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, 100, 50));
		assert_eq!(SettmintEngineModule::adjust_position(&ALICE, BTC, 0, -49).is_ok(), false);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, -100, -50));
	});
}

#[test]
fn liquidate_unsafe_settmint_by_reserve_auction() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, 100, 50));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 50);
		assert_eq!(SettersModule::positions(BTC, ALICE).standard, 50);
		assert_eq!(SettersModule::positions(BTC, ALICE).reserve, 100);
		assert_noop!(
			SettmintEngineModule::liquidate_unsafe_settmint(ALICE, BTC),
			Error::<Runtime>::MustBeUnsafe,
		);
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NoChange,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_ok!(SettmintEngineModule::liquidate_unsafe_settmint(ALICE, BTC));

		let liquidate_unsafe_settmint_event = Event::settmint_engine(crate::Event::LiquidateUnsafeSettmint(
			BTC,
			ALICE,
			100,
			50,
			LiquidationStrategy::Auction,
		));
		assert!(System::events()
			.iter()
			.any(|record| record.event == liquidate_unsafe_settmint_event));

		assert_eq!(SerpTreasuryModule::standard_pool(), 50);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 50);
		assert_eq!(SettersModule::positions(BTC, ALICE).standard, 0);
		assert_eq!(SettersModule::positions(BTC, ALICE).reserve, 0);

		mock_shutdown();
		assert_noop!(
			SettmintEngineModule::liquidate(Origin::none(), BTC, ALICE),
			Error::<Runtime>::AlreadyShutdown
		);
	});
}

#[test]
fn on_finalize_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			DOT,
			Change::NewValue(Some(Rate::saturating_from_rational(2, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		SettmintEngineModule::on_finalize(1);
		assert_eq!(SettmintEngineModule::standard_exchange_rate(BTC), None);
		assert_eq!(SettmintEngineModule::standard_exchange_rate(DOT), None);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, 100, 30));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 30);
		SettmintEngineModule::on_finalize(2);
		assert_eq!(
			SettmintEngineModule::standard_exchange_rate(BTC),
			Some(ExchangeRate::saturating_from_rational(101, 100))
		);
		assert_eq!(SettmintEngineModule::standard_exchange_rate(DOT), None);
		SettmintEngineModule::on_finalize(3);
		assert_eq!(
			SettmintEngineModule::standard_exchange_rate(BTC),
			Some(ExchangeRate::saturating_from_rational(10201, 10000))
		);
		assert_eq!(SettmintEngineModule::standard_exchange_rate(DOT), None);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, 0, -30));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USDJ, &ALICE), 0);
		SettmintEngineModule::on_finalize(4);
		assert_eq!(
			SettmintEngineModule::standard_exchange_rate(BTC),
			Some(ExchangeRate::saturating_from_rational(10201, 10000))
		);
		assert_eq!(SettmintEngineModule::standard_exchange_rate(DOT), None);
	});
}

#[test]
fn on_emergency_shutdown_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, 100, 30));
		SettmintEngineModule::on_finalize(1);
		assert_eq!(
			SettmintEngineModule::standard_exchange_rate(BTC),
			Some(ExchangeRate::saturating_from_rational(101, 100))
		);
		mock_shutdown();
		assert_eq!(<Runtime as Config>::EmergencyShutdown::is_shutdown(), true);
		SettmintEngineModule::on_finalize(2);
		assert_eq!(
			SettmintEngineModule::standard_exchange_rate(BTC),
			Some(ExchangeRate::saturating_from_rational(101, 100))
		);
	});
}

#[test]
fn settle_settmint_has_standard_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SettmintEngineModule::set_reserve_params(
			Origin::signed(1),
			BTC,
			Change::NewValue(Some(Rate::saturating_from_rational(1, 100000))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, 100, 0));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(SettersModule::positions(BTC, ALICE).standard, 0);
		assert_eq!(SettersModule::positions(BTC, ALICE).reserve, 100);
		assert_noop!(
			SettmintEngineModule::settle_settmint_has_standard(ALICE, BTC),
			Error::<Runtime>::NoStandardValue,
		);
		assert_ok!(SettmintEngineModule::adjust_position(&ALICE, BTC, 0, 50));
		assert_eq!(SettersModule::positions(BTC, ALICE).standard, 50);
		assert_eq!(SerpTreasuryModule::standard_pool(), 0);
		assert_eq!(SerpTreasuryModule::total_reserves(BTC), 0);
		assert_ok!(SettmintEngineModule::settle_settmint_has_standard(ALICE, BTC));

		let settle_settmint_in_standard_event = Event::settmint_engine(crate::Event::SettleSettmintInStandard(BTC, ALICE));
		assert!(System::events()
			.iter()
			.any(|record| record.event == settle_settmint_in_standard_event));

		assert_eq!(SettersModule::positions(BTC, ALICE).standard, 0);
		assert_eq!(SerpTreasuryModule::standard_pool(), 50);
		assert_eq!(SerpTreasuryModule::total_reserves(BTC), 50);

		assert_noop!(
			SettmintEngineModule::settle(Origin::none(), BTC, ALICE),
			Error::<Runtime>::MustAfterShutdown
		);
	});
}
