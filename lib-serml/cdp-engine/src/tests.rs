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

//! Unit tests for the cdp engine module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use orml_traits::MultiCurrency;
use sp_runtime::traits::BadOrigin;

#[test]
fn is_cdp_unsafe_work() {
	fn is_user_safe(collateral_currency_id: CurrencyId, stable_currency_id: CurrencyId, who: &AccountId) -> bool {
		let Position { collateral, debit } = LoansModule::setter_positions(collateral_currency_id, &who);
		CDPEngineModule::is_cdp_unsafe(collateral_currency_id, stable_currency_id, collateral, debit)
	}

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETR,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(is_user_safe(BTC, SETR, &ALICE), false);
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, BTC, SETR, 100, 50));
		assert_eq!(is_user_safe(BTC, SETR, &ALICE), false);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETR,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_eq!(is_user_safe(BTC, SETR, &ALICE), true);
	});
}

#[test]
fn get_debit_exchange_rate_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			CDPEngineModule::get_debit_exchange_rate(BTC, SETR),
			DefaultDebitExchangeRate::get()
		);
	});
}

#[test]
fn get_liquidation_penalty_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			CDPEngineModule::get_liquidation_penalty(BTC, SETUSD),
			DefaultLiquidationPenalty::get()
		);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETUSD,
			Change::NewValue(Some(Ratio::saturating_from_rational(5, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			CDPEngineModule::get_liquidation_penalty(BTC, SETUSD),
			Rate::saturating_from_rational(2, 10)
		);
	});
}

#[test]
fn get_liquidation_ratio_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			CDPEngineModule::get_liquidation_ratio(BTC, SETEUR),
			DefaultLiquidationRatio::get()
		);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETEUR,
			Change::NewValue(Some(Ratio::saturating_from_rational(5, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			CDPEngineModule::get_liquidation_ratio(BTC, SETEUR),
			Ratio::saturating_from_rational(5, 2)
		);
	});
}

#[test]
fn set_collateral_params_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			CDPEngineModule::set_collateral_params(
				Origin::signed(1),
				SETR,
				SETUSD,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
			),
			Error::<Runtime>::InvalidCollateralType
		);

		assert_noop!(
			CDPEngineModule::set_collateral_params(
				Origin::signed(1),
				BTC,
				DNAR,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
			),
			Error::<Runtime>::InvalidStableType
		);

		System::set_block_number(1);
		assert_noop!(
			CDPEngineModule::set_collateral_params(
				Origin::signed(5),
				BTC,
				SETUSD,
				Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
				Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
				Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
				Change::NewValue(10000),
			),
			BadOrigin
		);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETUSD,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		Event::cdp_engine(crate::Event::LiquidationRatioUpdated(
			BTC,
			SETUSD,
			Some(Ratio::saturating_from_rational(3, 2)),
		));
		Event::cdp_engine(crate::Event::LiquidationPenaltyUpdated(
			BTC,
			SETUSD,
			Some(Rate::saturating_from_rational(2, 10)),
		));
		Event::cdp_engine(crate::Event::RequiredCollateralRatioUpdated(
			BTC,
			SETUSD,
			Some(Ratio::saturating_from_rational(9, 5)),
		));
		Event::cdp_engine(crate::Event::MaximumTotalDebitValueUpdated(
			BTC, SETUSD, 10000,
		));

		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETUSD,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		let new_collateral_params = CDPEngineModule::collateral_params((BTC, SETUSD));

		assert_eq!(
			new_collateral_params.liquidation_ratio,
			Some(Ratio::saturating_from_rational(3, 2))
		);
		assert_eq!(
			new_collateral_params.liquidation_penalty,
			Some(Rate::saturating_from_rational(2, 10))
		);
		assert_eq!(
			new_collateral_params.required_collateral_ratio,
			Some(Ratio::saturating_from_rational(9, 5))
		);
		assert_eq!(new_collateral_params.maximum_total_debit_value, 10000);
	});
}

#[test]
fn calculate_collateral_ratio_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETGBP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			CDPEngineModule::calculate_collateral_ratio(BTC, SETGBP, 100, 50, Price::saturating_from_rational(1, 1)),
			Ratio::saturating_from_rational(100, 50)
		);
	});
}

#[test]
fn check_debit_cap_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETGBP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(CDPEngineModule::check_debit_cap(BTC, SETGBP, 9999));
		assert_noop!(
			CDPEngineModule::check_debit_cap(BTC, SETGBP, 10001),
			Error::<Runtime>::ExceedDebitValueHardCap,
		);
	});
}

#[test]
fn check_position_valid_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETEUR,
			Change::NewValue(Some(Ratio::saturating_from_rational(1, 1))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(10000),
		));

		MockPriceSource::set_relative_price(None);
		assert_noop!(
			CDPEngineModule::check_position_valid(BTC, SETEUR, 100, 50),
			Error::<Runtime>::InvalidFeedPrice
		);
		MockPriceSource::set_relative_price(Some(Price::one()));

		assert_ok!(CDPEngineModule::check_position_valid(BTC, SETEUR, 100, 50));
	});
}

#[test]
fn check_position_valid_failed_when_remain_debit_value_too_small() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETEUR,
			Change::NewValue(Some(Ratio::saturating_from_rational(1, 1))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(10000),
		));
		assert_noop!(
			CDPEngineModule::check_position_valid(BTC, SETEUR, 2, 1),
			Error::<Runtime>::RemainDebitValueTooSmall,
		);
	});
}

#[test]
fn check_position_valid_ratio_below_liquidate_ratio() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETR,
			Change::NewValue(Some(Ratio::saturating_from_rational(10, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			CDPEngineModule::check_position_valid(BTC, SETR, 91, 50),
			Error::<Runtime>::BelowLiquidationRatio,
		);
	});
}

#[test]
fn check_position_valid_ratio_below_required_ratio() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETR,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			CDPEngineModule::check_position_valid(BTC, SETR, 89, 50),
			Error::<Runtime>::BelowRequiredCollateralRatio
		);
	});
}

#[test]
fn adjust_position_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETUSD,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			CDPEngineModule::adjust_position(&ALICE, SETM, SETUSD, 100, 50),
			Error::<Runtime>::InvalidCollateralType,
		);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 0);
		assert_eq!(LoansModule::setdollar_positions(BTC, ALICE).debit, 0);
		assert_eq!(LoansModule::setdollar_positions(BTC, ALICE).collateral, 0);
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, BTC, SETUSD, 100, 50));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
		assert_eq!(LoansModule::setdollar_positions(BTC, ALICE).debit, 50);
		assert_eq!(LoansModule::setdollar_positions(BTC, ALICE).collateral, 100);
		assert_eq!(CDPEngineModule::adjust_position(&ALICE, BTC, SETUSD, 0, 20).is_ok(), false);
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, BTC, SETUSD, 0, -20));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 30);
		assert_eq!(LoansModule::setdollar_positions(BTC, ALICE).debit, 30);
		assert_eq!(LoansModule::setdollar_positions(BTC, ALICE).collateral, 100);
	});
}

#[test]
fn remain_debit_value_too_small_check() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETUSD,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, BTC, SETUSD, 100, 50));
		assert_eq!(CDPEngineModule::adjust_position(&ALICE, BTC, SETUSD, 0, -49).is_ok(), false);
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, BTC, SETUSD, -100, -50));
	});
}

#[test]
fn liquidate_unsafe_cdp_by_collateral_auction() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETEUR,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, BTC, SETEUR, 100, 50));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETEUR, &ALICE), 50);
		assert_eq!(LoansModule::seteuro_positions(BTC, ALICE).debit, 50);
		assert_eq!(LoansModule::seteuro_positions(BTC, ALICE).collateral, 100);
		assert_noop!(
			CDPEngineModule::liquidate_unsafe_cdp(ALICE, BTC, SETEUR, None),
			Error::<Runtime>::MustBeUnsafe,
		);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETEUR,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_ok!(CDPEngineModule::liquidate_unsafe_cdp(ALICE, BTC, SETEUR, None));
		Event::cdp_engine(crate::Event::LiquidateUnsafeCDP(
			BTC,
			SETEUR,
			ALICE,
			100,
			50,
			LiquidationStrategy::Auction,
		));
		assert_eq!(CDPTreasuryModule::debit_pool(SETEUR), 50);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETEUR, &ALICE), 50);
		assert_eq!(LoansModule::seteuro_positions(BTC, ALICE).debit, 0);
		assert_eq!(LoansModule::seteuro_positions(BTC, ALICE).collateral, 0);

		mock_shutdown();
		assert_noop!(
			CDPEngineModule::liquidate(Origin::none(), BTC, SETEUR, ALICE, None),
			Error::<Runtime>::AlreadyShutdown
		);
	});
}

#[test]
fn settle_cdp_has_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETGBP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, BTC, SETGBP, 100, 0));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(LoansModule::setpound_positions(BTC, ALICE).debit, 0);
		assert_eq!(LoansModule::setpound_positions(BTC, ALICE).collateral, 100);
		assert_noop!(
			CDPEngineModule::settle_cdp_has_debit(ALICE, BTC, SETGBP),
			Error::<Runtime>::NoDebitValue,
		);
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, BTC, SETGBP, 0, 50));
		assert_eq!(LoansModule::setpound_positions(BTC, ALICE).debit, 50);
		assert_eq!(CDPTreasuryModule::debit_pool(SETGBP), 0);
		assert_eq!(CDPTreasuryModule::total_collaterals(BTC), 0);
		assert_ok!(CDPEngineModule::settle_cdp_has_debit(ALICE, BTC, SETGBP));
		Event::cdp_engine(crate::Event::SettleCDPInDebit(BTC, SETGBP, ALICE));
		assert_eq!(LoansModule::setpound_positions(BTC, ALICE).debit, 0);
		assert_eq!(CDPTreasuryModule::debit_pool(SETGBP), 50);
		assert_eq!(CDPTreasuryModule::total_collaterals(BTC), 50);

		assert_noop!(
			CDPEngineModule::settle(Origin::none(), BTC, SETGBP, ALICE),
			Error::<Runtime>::MustAfterShutdown
		);
	});
}

#[test]
fn close_cdp_has_debit_by_dex_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(DEXModule::add_liquidity(
			Origin::signed(CAROL),
			BTC,
			SETR,
			100,
			1000,
			0,
		));
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETR,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		assert_ok!(CDPEngineModule::adjust_position(&ALICE, BTC, SETR, 100, 0));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 0);
		assert_eq!(LoansModule::setter_positions(BTC, ALICE).debit, 0);
		assert_eq!(LoansModule::setter_positions(BTC, ALICE).collateral, 100);

		assert_noop!(
			CDPEngineModule::close_cdp_has_debit_by_dex(ALICE, BTC, SETR, None),
			Error::<Runtime>::NoDebitValue
		);

		assert_ok!(CDPEngineModule::adjust_position(&ALICE, BTC, SETR, 0, 50));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 50);
		assert_eq!(LoansModule::setter_positions(BTC, ALICE).debit, 50);
		assert_eq!(LoansModule::setter_positions(BTC, ALICE).collateral, 100);
		assert_eq!(CDPTreasuryModule::get_surplus_pool(SETR), 0);
		assert_eq!(CDPTreasuryModule::get_debit_pool(SETR), 0);

		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETR,
			Change::NewValue(Some(Ratio::saturating_from_rational(5, 2))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_noop!(
			CDPEngineModule::close_cdp_has_debit_by_dex(ALICE, BTC, SETR, None),
			Error::<Runtime>::IsUnsafe
		);

		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			BTC,
			SETR,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_ok!(CDPEngineModule::close_cdp_has_debit_by_dex(ALICE, BTC, SETR, None));
		Event::cdp_engine(crate::Event::CloseCDPInDebitByDEX(
			BTC, SETR, ALICE, 6, 94, 50,
		));
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 994);
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 50);
		assert_eq!(LoansModule::setter_positions(BTC, ALICE).debit, 0);
		assert_eq!(LoansModule::setter_positions(BTC, ALICE).collateral, 0);
		assert_eq!(CDPTreasuryModule::get_surplus_pool(SETR), 50);
		assert_eq!(CDPTreasuryModule::get_debit_pool(SETR), 50);
	});
}
