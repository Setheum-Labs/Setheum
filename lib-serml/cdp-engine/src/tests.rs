// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

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
use mock::{Call as MockCall, Event, *};
use orml_traits::MultiCurrency;
use sp_core::offchain::{testing, OffchainDbExt, OffchainWorkerExt, TransactionPoolExt};
use sp_io::offchain;
use sp_runtime::{
	offchain::{DbExternalities, StorageKind},
	traits::BadOrigin,
};
use support::DEXManager;

pub const INIT_TIMESTAMP: u64 = 30_000;
pub const BLOCK_TIME: u64 = 1000;

fn run_to_block_offchain(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
		Timestamp::set_timestamp((System::block_number() as u64 * BLOCK_TIME) + INIT_TIMESTAMP);
		CDPEngineModule::on_initialize(System::block_number());
		CDPEngineModule::offchain_worker(System::block_number());
		// this unlocks the concurrency storage lock so offchain_worker will fire next block
		offchain::sleep_until(offchain::timestamp().add(Duration::from_millis(LOCK_DURATION + 200)));
	}
}

#[test]
fn check_cdp_status_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(CDPEngineModule::check_cdp_status(SERP, 100, 500), CDPStatus::Safe);

		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_eq!(CDPEngineModule::check_cdp_status(SERP, 100, 500), CDPStatus::Unsafe);

		MockPriceSource::set_price(SERP, None);
		assert_eq!(
			CDPEngineModule::check_cdp_status(SERP, 100, 500),
			CDPStatus::ChecksFailed(Error::<Runtime>::InvalidFeedPrice.into())
		);
	});
}

#[test]
fn get_debit_exchange_rate_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(CDPEngineModule::debit_exchange_rate(SERP), None);
		assert_eq!(
			CDPEngineModule::get_debit_exchange_rate(SERP),
			ExchangeRate::saturating_from_rational(1, 10)
		);

		DebitExchangeRate::<Runtime>::insert(SERP, ExchangeRate::one());
		assert_eq!(CDPEngineModule::debit_exchange_rate(SERP), Some(ExchangeRate::one()));
		assert_eq!(CDPEngineModule::get_debit_exchange_rate(SERP), ExchangeRate::one());
	});
}

#[test]
fn get_liquidation_penalty_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			CDPEngineModule::get_liquidation_penalty(SERP),
			DefaultLiquidationPenalty::get()
		);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(5, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			CDPEngineModule::get_liquidation_penalty(SERP),
			Rate::saturating_from_rational(2, 10)
		);
	});
}

#[test]
fn get_liquidation_ratio_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			CDPEngineModule::get_liquidation_ratio(SERP),
			DefaultLiquidationRatio::get()
		);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(5, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			CDPEngineModule::get_liquidation_ratio(SERP),
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
				SETUSD,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
				Change::NoChange,
			),
			Error::<Runtime>::InvalidCollateralType
		);

		System::set_block_number(1);
		assert_noop!(
			CDPEngineModule::set_collateral_params(
				Origin::signed(5),
				SERP,
				Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
				Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
				Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
				Change::NewValue(10000),
			),
			BadOrigin
		);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		System::assert_has_event(Event::CDPEngineModule(crate::Event::LiquidationRatioUpdated {
			collateral_type: SERP,
			new_liquidation_ratio: Some(Ratio::saturating_from_rational(3, 2)),
		}));
		System::assert_has_event(Event::CDPEngineModule(crate::Event::LiquidationPenaltyUpdated {
			collateral_type: SERP,
			new_liquidation_penalty: Some(Rate::saturating_from_rational(2, 10)),
		}));
		System::assert_has_event(Event::CDPEngineModule(crate::Event::RequiredCollateralRatioUpdated {
			collateral_type: SERP,
			new_required_collateral_ratio: Some(Ratio::saturating_from_rational(9, 5)),
		}));
		System::assert_has_event(Event::CDPEngineModule(crate::Event::MaximumTotalDebitValueUpdated {
			collateral_type: SERP,
			new_total_debit_value: 10000,
		}));

		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		let new_collateral_params = CDPEngineModule::collateral_params(SERP);

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
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_eq!(
			CDPEngineModule::calculate_collateral_ratio(SERP, 1000, 50, Price::saturating_from_rational(1, 1)),
			Ratio::saturating_from_integer(200)
		);
	});
}

#[test]
fn check_debit_cap_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(CDPEngineModule::check_debit_cap(SERP, 100000));
		assert_noop!(
			CDPEngineModule::check_debit_cap(SERP, 100010),
			Error::<Runtime>::ExceedDebitValueHardCap,
		);
	});
}

#[test]
fn check_position_valid_failed_when_invalid_feed_price() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(1, 1))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(10000),
		));

		MockPriceSource::set_price(SERP, None);
		assert_noop!(
			CDPEngineModule::check_position_valid(SERP, 100, 50, true),
			Error::<Runtime>::InvalidFeedPrice
		);

		MockPriceSource::set_price(SERP, Some(Price::one()));
		assert_ok!(CDPEngineModule::check_position_valid(SERP, 100, 50, true));
	});
}

#[test]
fn check_position_valid_failed_when_remain_debit_value_too_small() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(1, 1))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(10000),
		));
		assert_noop!(
			CDPEngineModule::check_position_valid(SERP, 2, 1, true),
			Error::<Runtime>::RemainDebitValueTooSmall,
		);
	});
}

#[test]
fn check_position_valid_ratio_below_liquidate_ratio() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(10, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			CDPEngineModule::check_position_valid(SERP, 9, 5, true),
			Error::<Runtime>::RemainDebitValueTooSmall,
		);
	});
}

#[test]
fn check_position_valid_ratio_below_required_ratio() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(CDPEngineModule::check_position_valid(SERP, 89, 500, false));
	});
}

#[test]
fn adjust_position_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_noop!(
			CDPEngineModule::adjust_position(&ALICE, SETM, 100, 500),
			Error::<Runtime>::InvalidCollateralType,
		);
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 0);
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 100, 500));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 500);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);
		assert!(!CDPEngineModule::adjust_position(&ALICE, SERP, 0, 200).is_ok());
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 0, -200));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 30);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 300);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);
	});
}

// #[test]
// fn expand_position_collateral_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		MockPriceSource::set_price(DNAR, Some(Price::saturating_from_rational(10, 1)));
// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			DNAR,
// 			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
// 			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
// 			Change::NewValue(Some(Ratio::saturating_from_rational(2, 1))),
// 			Change::NewValue(10000),
// 		));
// 		assert_ok!(CDPEngineModule::adjust_position(&ALICE, DNAR, 100, 2500));
// 		assert_eq!(
// 			LoansModule::positions(DNAR, ALICE),
// 			Position {
// 				collateral: 100,
// 				debit: 2500
// 			}
// 		);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 900);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 250);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
// 		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 100);
// 		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 0);

// 		assert_noop!(
// 			CDPEngineModule::expand_position_collateral(&ALICE, DNAR, 0, 1),
// 			Error::<Runtime>::CannotSwap
// 		);

// 		assert_ok!(DEXModule::add_liquidity(
// 			Origin::signed(CAROL),
// 			SETUSD,
// 			DNAR,
// 			10000,
// 			1000,
// 			0
// 		));
// 		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (1000, 10000));
// 		assert_noop!(
// 			CDPEngineModule::expand_position_collateral(&ALICE, DNAR, 250, 100),
// 			Error::<Runtime>::CannotSwap
// 		);

// 		assert_ok!(CDPEngineModule::expand_position_collateral(&ALICE, DNAR, 250, 20));
// 		assert_eq!(
// 			LoansModule::positions(DNAR, ALICE),
// 			Position {
// 				collateral: 124,
// 				debit: 5000
// 			}
// 		);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 900);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 250);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
// 		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 124);
// 		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 0);
// 		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (976, 10250));

// 		assert_noop!(
// 			CDPEngineModule::expand_position_collateral(&ALICE, DNAR, 300, 0),
// 			Error::<Runtime>::BelowRequiredCollateralRatio
// 		);

// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			DNAR,
// 			Change::NoChange,
// 			Change::NoChange,
// 			Change::NoChange,
// 			Change::NewValue(700),
// 		));
// 		assert_noop!(
// 			CDPEngineModule::expand_position_collateral(&ALICE, DNAR, 201, 0),
// 			Error::<Runtime>::ExceedDebitValueHardCap
// 		);

// 		assert_ok!(CDPEngineModule::expand_position_collateral(&ALICE, DNAR, 200, 18));
// 		assert_eq!(
// 			LoansModule::positions(DNAR, ALICE),
// 			Position {
// 				collateral: 142,
// 				debit: 7000
// 			}
// 		);
// 		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 142);
// 		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 0);
// 		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (958, 10450));
// 	});
// }

// #[test]
// fn expand_position_collateral_for_lp_setusd_dnar_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_ok!(DEXModule::add_liquidity(
// 			Origin::signed(CAROL),
// 			SETUSD,
// 			DNAR,
// 			10000,
// 			1000,
// 			0
// 		));
// 		assert_eq!(Currencies::total_issuance(LP_SETUSD_DNAR), 20000);
// 		assert_ok!(Currencies::transfer(Origin::signed(CAROL), ALICE, LP_SETUSD_DNAR, 1000));

// 		MockPriceSource::set_price(LP_SETUSD_DNAR, Some(Price::saturating_from_rational(1, 1)));
// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			LP_SETUSD_DNAR,
// 			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
// 			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
// 			Change::NewValue(Some(Ratio::saturating_from_rational(2, 1))),
// 			Change::NewValue(10000),
// 		));
// 		assert_ok!(CDPEngineModule::adjust_position(&ALICE, LP_SETUSD_DNAR, 1000, 2000));
// 		assert_eq!(
// 			LoansModule::positions(LP_SETUSD_DNAR, ALICE),
// 			Position {
// 				collateral: 1000,
// 				debit: 2000
// 			}
// 		);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 200);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
// 		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 1000);
// 		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 0);
// 		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 0);
// 		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (1000, 10000));

// 		assert_noop!(
// 			CDPEngineModule::expand_position_collateral(&ALICE, LP_SETUSD_DNAR, 200, 200),
// 			dex::Error::<Runtime>::UnacceptableShareIncrement
// 		);

// 		assert_ok!(CDPEngineModule::expand_position_collateral(
// 			&ALICE,
// 			LP_SETUSD_DNAR,
// 			300,
// 			100
// 		));
// 		assert_eq!(
// 			LoansModule::positions(LP_SETUSD_DNAR, ALICE),
// 			Position {
// 				collateral: 1283,
// 				debit: 5000
// 			}
// 		);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 206);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
// 		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 1283);
// 		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 0);
// 		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 0);
// 		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (1000, 10294));
// 	});
// }

#[test]
fn shrink_position_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		MockPriceSource::set_price(DNAR, Some(Price::saturating_from_rational(10, 1)));
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			DNAR,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(2, 1))),
			Change::NewValue(10000),
		));
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, DNAR, 100, 5000));
		assert_eq!(
			LoansModule::positions(DNAR, ALICE),
			Position {
				collateral: 100,
				debit: 5000
			}
		);
		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 500);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 100);
		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 0);

		MockPriceSource::set_price(DNAR, Some(Price::saturating_from_rational(8, 1)));
		assert_noop!(
			CDPEngineModule::shrink_position_debit(&ALICE, DNAR, 10, 0),
			Error::<Runtime>::CannotSwap
		);

		assert_ok!(DEXModule::add_liquidity(
			Origin::signed(CAROL),
			SETUSD,
			DNAR,
			8000,
			1000,
			0
		));
		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (1000, 8000));
		assert_noop!(
			CDPEngineModule::shrink_position_debit(&ALICE, DNAR, 10, 80),
			Error::<Runtime>::CannotSwap
		);

		assert_ok!(CDPEngineModule::shrink_position_debit(&ALICE, DNAR, 10, 70));
		assert_eq!(
			LoansModule::positions(DNAR, ALICE),
			Position {
				collateral: 90,
				debit: 4220
			}
		);
		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 500);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 90);
		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 0);
		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (1010, 7922));

		assert_ok!(CDPEngineModule::shrink_position_debit(&ALICE, DNAR, 70, 0));
		assert_eq!(
			LoansModule::positions(DNAR, ALICE),
			Position {
				collateral: 20,
				debit: 0
			}
		);
		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 586);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 20);
		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 3);
		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (1080, 7414));
	});
}

// #[test]
// fn shrink_position_debit_for_lp_setusd_dnar_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		MockPriceSource::set_price(LP_SETUSD_DNAR, Some(Price::saturating_from_rational(1, 1)));
// 		assert_ok!(DEXModule::add_liquidity(
// 			Origin::signed(CAROL),
// 			SETUSD,
// 			DNAR,
// 			10000,
// 			1000,
// 			0
// 		));
// 		assert_eq!(Currencies::total_issuance(LP_SETUSD_DNAR), 20000);
// 		assert_ok!(Currencies::transfer(Origin::signed(CAROL), ALICE, LP_SETUSD_DNAR, 1000));

// 		MockPriceSource::set_price(LP_SETUSD_DNAR, Some(Price::saturating_from_rational(1, 1)));
// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			LP_SETUSD_DNAR,
// 			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
// 			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
// 			Change::NewValue(Some(Ratio::saturating_from_rational(2, 1))),
// 			Change::NewValue(10000),
// 		));
// 		assert_ok!(CDPEngineModule::adjust_position(&ALICE, LP_SETUSD_DNAR, 1000, 5000));
// 		assert_eq!(
// 			LoansModule::positions(LP_SETUSD_DNAR, ALICE),
// 			Position {
// 				collateral: 1000,
// 				debit: 5000
// 			}
// 		);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 500);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
// 		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 1000);
// 		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 0);
// 		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 0);
// 		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (1000, 10000));

// 		assert_noop!(
// 			CDPEngineModule::shrink_position_debit(&ALICE, LP_SETUSD_DNAR, 200, 200),
// 			Error::<Runtime>::NotEnoughDebitDecrement
// 		);

// 		assert_ok!(CDPEngineModule::shrink_position_debit(&ALICE, LP_SETUSD_DNAR, 100, 80));
// 		assert_eq!(
// 			LoansModule::positions(LP_SETUSD_DNAR, ALICE),
// 			Position {
// 				collateral: 900,
// 				debit: 4010
// 			}
// 		);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 500);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
// 		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 900);
// 		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 0);
// 		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 0);
// 		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (1000, 9901));

// 		assert_ok!(CDPEngineModule::shrink_position_debit(&ALICE, LP_SETUSD_DNAR, 600, 500));
// 		assert_eq!(
// 			LoansModule::positions(LP_SETUSD_DNAR, ALICE),
// 			Position {
// 				collateral: 300,
// 				debit: 0
// 			}
// 		);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 685);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
// 		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 300);
// 		assert_eq!(Currencies::free_balance(DNAR, &LoansModule::account_id()), 0);
// 		assert_eq!(Currencies::free_balance(SETUSD, &LoansModule::account_id()), 0);
// 		assert_eq!(DEXModule::get_liquidity_pool(DNAR, SETUSD), (1000, 9315));
// 	});
// }

#[test]
fn remain_debit_value_too_small_check() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 100, 500));
		assert!(!CDPEngineModule::adjust_position(&ALICE, SERP, 0, -490).is_ok());
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, -100, -500));
	});
}

// #[test]
// fn liquidate_unsafe_cdp_by_collateral_auction() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);
// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			SERP,
// 			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
// 			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
// 			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
// 			Change::NewValue(10000),
// 		));
// 		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 100, 50));
// 		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 5);
// 		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 50);
// 		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);
// 		assert_noop!(
// 			CDPEngineModule::liquidate_unsafe_cdp(ALICE, SERP),
// 			Error::<Runtime>::MustBeUnsafe,
// 		);
// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			SERP,
// 			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
// 			Change::NoChange,
// 			Change::NoChange,
// 			Change::NoChange,
// 		));
// 		assert_ok!(CDPEngineModule::liquidate_unsafe_cdp(ALICE, SERP));

// 		System::assert_last_event(Event::CDPEngineModule(crate::Event::LiquidateUnsafeCDP {
// 			collateral_type: SERP,
// 			owner: ALICE,
// 			collateral_amount: 100,
// 			bad_debt_value: 50,
// 			target_amount: 60,
// 		}));
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 50);
// 		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
// 		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
// 		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 0);

// 		mock_shutdown();
// 		assert_noop!(
// 			CDPEngineModule::liquidate(Origin::none(), SERP, ALICE),
// 			Error::<Runtime>::AlreadyShutdown
// 		);
// 	});
// }

#[test]
fn liquidate_unsafe_cdp_by_collateral_auction_when_limited_by_slippage() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(DEXModule::add_liquidity(
			Origin::signed(CAROL),
			SERP,
			SETUSD,
			100,
			121,
			0
		));
		assert_eq!(DEXModule::get_liquidity_pool(SERP, SETUSD), (100, 121));

		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 100, 500));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 500);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);

		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::max_value())),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));

		// pool is enough, but slippage limit the swap
		MockPriceSource::set_price(SERP, Some(Price::saturating_from_rational(2, 1)));
		assert_eq!(
			DEXModule::get_swap_amount(&vec![SERP, SETUSD], SwapLimit::ExactTarget(Balance::MAX, 60)),
			Some((99, 60))
		);
		assert_eq!(
			DEXModule::get_swap_amount(&vec![SERP, SETUSD], SwapLimit::ExactSupply(100, 0)),
			Some((100, 60))
		);
		assert_ok!(CDPEngineModule::liquidate_unsafe_cdp(ALICE, SERP));
		System::assert_last_event(Event::CDPEngineModule(crate::Event::LiquidateUnsafeCDP {
			collateral_type: SERP,
			owner: ALICE,
			collateral_amount: 100,
			bad_debt_value: 50,
			target_amount: 60,
		}));

		assert_eq!(DEXModule::get_liquidity_pool(SERP, SETUSD), (100, 121));
		assert_eq!(CDPTreasuryModule::debit_pool(), 50);
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 0);
	});
}

#[test]
fn liquidate_unsafe_cdp_by_swap() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(DEXModule::add_liquidity(
			Origin::signed(CAROL),
			SERP,
			SETUSD,
			100,
			121,
			0
		));
		assert_eq!(DEXModule::get_liquidity_pool(SERP, SETUSD), (100, 121));

		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 100, 500));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 500);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);

		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::max_value())),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));

		assert_ok!(CDPEngineModule::liquidate_unsafe_cdp(ALICE, SERP));
		System::assert_last_event(Event::CDPEngineModule(crate::Event::LiquidateUnsafeCDP {
			collateral_type: SERP,
			owner: ALICE,
			collateral_amount: 100,
			bad_debt_value: 50,
			target_amount: 60,
		}));

		assert_eq!(DEXModule::get_liquidity_pool(SERP, SETUSD), (199, 61));
		assert_eq!(CDPTreasuryModule::debit_pool(), 50);
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 0);
	});
}

// #[test]
// fn liquidate_unsafe_cdp_of_lp_setusd_dnar_and_swap_dnar() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);
// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			LP_SETUSD_DNAR,
// 			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
// 			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
// 			Change::NewValue(Some(Ratio::saturating_from_rational(2, 1))),
// 			Change::NewValue(10000),
// 		));
// 		assert_ok!(DEXModule::add_liquidity(
// 			Origin::signed(CAROL),
// 			SETUSD,
// 			DNAR,
// 			10000,
// 			500,
// 			0
// 		));
// 		assert_eq!(DEXModule::get_liquidity_pool(SETUSD, DNAR), (10000, 500));
// 		assert_eq!(Currencies::total_issuance(LP_SETUSD_DNAR), 20000);
// 		assert_ok!(Currencies::transfer(Origin::signed(CAROL), ALICE, LP_SETUSD_DNAR, 1000));
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 0);

// 		MockPriceSource::set_price(DNAR, Price::checked_from_rational(20, 1));
// 		MockPriceSource::set_price(LP_SETUSD_DNAR, Price::checked_from_rational(1, 1));

// 		assert_ok!(CDPEngineModule::adjust_position(&ALICE, LP_SETUSD_DNAR, 1000, 5000));
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 500);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).debit, 5000);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).collateral, 1000);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 1000);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
// 		assert_eq!(Currencies::free_balance(SETUSD, &CDPTreasuryModule::account_id()), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &CDPTreasuryModule::account_id()), 0);
// 		assert_eq!(
// 			Currencies::free_balance(LP_SETUSD_DNAR, &CDPTreasuryModule::account_id()),
// 			0
// 		);
// 		assert_eq!(AUCTION.with(|v| *v.borrow()), None);

// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			LP_SETUSD_DNAR,
// 			Change::NewValue(Some(Ratio::max_value())),
// 			Change::NoChange,
// 			Change::NoChange,
// 			Change::NoChange,
// 		));

// 		assert_ok!(CDPEngineModule::liquidate_unsafe_cdp(ALICE, LP_SETUSD_DNAR));
// 		System::assert_last_event(Event::CDPEngineModule(crate::Event::LiquidateUnsafeCDP {
// 			collateral_type: LP_SETUSD_DNAR,
// 			owner: ALICE,
// 			collateral_amount: 1000,
// 			bad_debt_value: 500,
// 			target_amount: 600,
// 		}));

// 		assert_eq!(
// 			MockPriceSource::get_relative_price(SETUSD, DNAR),
// 			Price::checked_from_rational(1, 20)
// 		);
// 		assert_eq!(DEXModule::get_liquidity_pool(SETUSD, DNAR), (9400, 481));
// 		assert_eq!(Currencies::total_issuance(LP_SETUSD_DNAR), 19000);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1019);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 500);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).debit, 0);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).collateral, 0);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 0);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 500);
// 		assert_eq!(Currencies::free_balance(SETUSD, &CDPTreasuryModule::account_id()), 600);
// 		assert_eq!(Currencies::free_balance(DNAR, &CDPTreasuryModule::account_id()), 0);
// 		assert_eq!(
// 			Currencies::free_balance(LP_SETUSD_DNAR, &CDPTreasuryModule::account_id()),
// 			0
// 		);
// 		assert_eq!(AUCTION.with(|v| *v.borrow()), None);
// 	});
// }

// #[test]
// fn liquidate_unsafe_cdp_of_lp_setusd_dnar_and_setusd_take_whole_target() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);
// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			LP_SETUSD_DNAR,
// 			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
// 			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
// 			Change::NewValue(Some(Ratio::saturating_from_rational(2, 1))),
// 			Change::NewValue(10000),
// 		));
// 		assert_ok!(DEXModule::add_liquidity(
// 			Origin::signed(CAROL),
// 			SETUSD,
// 			DNAR,
// 			10000,
// 			500,
// 			0
// 		));
// 		assert_eq!(DEXModule::get_liquidity_pool(SETUSD, DNAR), (10000, 500));
// 		assert_eq!(Currencies::total_issuance(LP_SETUSD_DNAR), 20000);
// 		assert_ok!(Currencies::transfer(Origin::signed(CAROL), ALICE, LP_SETUSD_DNAR, 1000));
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 0);

// 		MockPriceSource::set_price(DNAR, Price::checked_from_rational(20, 1));
// 		MockPriceSource::set_price(LP_SETUSD_DNAR, Price::checked_from_rational(1, 1));

// 		assert_ok!(CDPEngineModule::adjust_position(&ALICE, LP_SETUSD_DNAR, 1000, 2000));
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 200);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).debit, 2000);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).collateral, 1000);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 1000);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
// 		assert_eq!(Currencies::free_balance(SETUSD, &CDPTreasuryModule::account_id()), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &CDPTreasuryModule::account_id()), 0);
// 		assert_eq!(
// 			Currencies::free_balance(LP_SETUSD_DNAR, &CDPTreasuryModule::account_id()),
// 			0
// 		);
// 		assert_eq!(AUCTION.with(|v| *v.borrow()), None);

// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			LP_SETUSD_DNAR,
// 			Change::NewValue(Some(Ratio::max_value())),
// 			Change::NoChange,
// 			Change::NoChange,
// 			Change::NoChange,
// 		));

// 		assert_ok!(CDPEngineModule::liquidate_unsafe_cdp(ALICE, LP_SETUSD_DNAR));
// 		System::assert_last_event(Event::CDPEngineModule(crate::Event::LiquidateUnsafeCDP {
// 			collateral_type: LP_SETUSD_DNAR,
// 			owner: ALICE,
// 			collateral_amount: 1000,
// 			bad_debt_value: 200,
// 			target_amount: 240,
// 		}));

// 		assert_eq!(
// 			MockPriceSource::get_relative_price(SETUSD, DNAR),
// 			Price::checked_from_rational(1, 20)
// 		);
// 		assert_eq!(DEXModule::get_liquidity_pool(SETUSD, DNAR), (9500, 475));
// 		assert_eq!(Currencies::total_issuance(LP_SETUSD_DNAR), 19000);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1025);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 460);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).debit, 0);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).collateral, 0);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 0);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 200);
// 		assert_eq!(Currencies::free_balance(SETUSD, &CDPTreasuryModule::account_id()), 240);
// 		assert_eq!(Currencies::free_balance(DNAR, &CDPTreasuryModule::account_id()), 0);
// 		assert_eq!(
// 			Currencies::free_balance(LP_SETUSD_DNAR, &CDPTreasuryModule::account_id()),
// 			0
// 		);
// 		assert_eq!(AUCTION.with(|v| *v.borrow()), None);
// 	});
// }

// #[test]
// fn liquidate_unsafe_cdp_of_lp_setusd_dnar_and_create_dnar_auction() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);
// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			LP_SETUSD_DNAR,
// 			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
// 			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
// 			Change::NewValue(Some(Ratio::saturating_from_rational(2, 1))),
// 			Change::NewValue(10000),
// 		));
// 		assert_ok!(DEXModule::add_liquidity(
// 			Origin::signed(CAROL),
// 			SETUSD,
// 			DNAR,
// 			500,
// 			25,
// 			0
// 		));
// 		assert_eq!(DEXModule::get_liquidity_pool(SETUSD, DNAR), (500, 25));
// 		assert_eq!(Currencies::total_issuance(LP_SETUSD_DNAR), 1000);
// 		assert_ok!(Currencies::transfer(Origin::signed(CAROL), ALICE, LP_SETUSD_DNAR, 1000));
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 0);

// 		MockPriceSource::set_price(DNAR, Price::checked_from_rational(20, 1));
// 		MockPriceSource::set_price(LP_SETUSD_DNAR, Price::checked_from_rational(1, 1));

// 		assert_ok!(CDPEngineModule::adjust_position(&ALICE, LP_SETUSD_DNAR, 1000, 5000));
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 500);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).debit, 5000);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).collateral, 1000);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 1000);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
// 		assert_eq!(Currencies::free_balance(SETUSD, &CDPTreasuryModule::account_id()), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &CDPTreasuryModule::account_id()), 0);
// 		assert_eq!(
// 			Currencies::free_balance(LP_SETUSD_DNAR, &CDPTreasuryModule::account_id()),
// 			0
// 		);
// 		assert_eq!(AUCTION.with(|v| *v.borrow()), None);

// 		assert_ok!(CDPEngineModule::set_collateral_params(
// 			Origin::signed(1),
// 			LP_SETUSD_DNAR,
// 			Change::NewValue(Some(Ratio::max_value())),
// 			Change::NoChange,
// 			Change::NoChange,
// 			Change::NoChange,
// 		));

// 		assert_ok!(CDPEngineModule::liquidate_unsafe_cdp(ALICE, LP_SETUSD_DNAR));
// 		System::assert_last_event(Event::CDPEngineModule(crate::Event::LiquidateUnsafeCDP {
// 			collateral_type: LP_SETUSD_DNAR,
// 			owner: ALICE,
// 			collateral_amount: 1000,
// 			bad_debt_value: 500,
// 			target_amount: 600,
// 		}));

// 		assert_eq!(
// 			MockPriceSource::get_relative_price(SETUSD, DNAR),
// 			Price::checked_from_rational(1, 20)
// 		);
// 		assert_eq!(DEXModule::get_liquidity_pool(SETUSD, DNAR), (0, 0));
// 		assert_eq!(Currencies::total_issuance(LP_SETUSD_DNAR), 0);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &ALICE), 0);
// 		assert_eq!(Currencies::free_balance(DNAR, &ALICE), 1000);
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 500);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).debit, 0);
// 		assert_eq!(LoansModule::positions(LP_SETUSD_DNAR, ALICE).collateral, 0);
// 		assert_eq!(Currencies::free_balance(LP_SETUSD_DNAR, &LoansModule::account_id()), 0);
// 		assert_eq!(CDPTreasuryModule::debit_pool(), 500);
// 		assert_eq!(Currencies::free_balance(SETUSD, &CDPTreasuryModule::account_id()), 500);
// 		assert_eq!(Currencies::free_balance(DNAR, &CDPTreasuryModule::account_id()), 25);
// 		assert_eq!(
// 			Currencies::free_balance(LP_SETUSD_DNAR, &CDPTreasuryModule::account_id()),
// 			0
// 		);
// 		assert_eq!(AUCTION.with(|v| *v.borrow()), Some((ALICE, DNAR, 25, 100)));
// 	});
// }

#[test]
fn settle_cdp_has_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 100, 0));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);
		assert_noop!(
			CDPEngineModule::settle_cdp_has_debit(ALICE, SERP),
			Error::<Runtime>::NoDebitValue,
		);
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 0, 500));
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 500);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
		assert_eq!(CDPTreasuryModule::total_collaterals(SERP), 0);
		assert_ok!(CDPEngineModule::settle_cdp_has_debit(ALICE, SERP));
		System::assert_last_event(Event::CDPEngineModule(crate::Event::SettleCDPInDebit {
			collateral_type: SERP,
			owner: ALICE,
		}));
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
		assert_eq!(CDPTreasuryModule::debit_pool(), 50);
		assert_eq!(CDPTreasuryModule::total_collaterals(SERP), 50);

		assert_noop!(
			CDPEngineModule::settle(Origin::none(), SERP, ALICE),
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
			SERP,
			SETUSD,
			100,
			1000,
			0
		));
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 100, 0));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);

		assert_noop!(
			CDPEngineModule::close_cdp_has_debit_by_dex(ALICE, SERP, 100),
			Error::<Runtime>::NoDebitValue
		);

		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 0, 500));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 500);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);
		assert_eq!(CDPTreasuryModule::get_surplus_pool(), 0);
		assert_eq!(CDPTreasuryModule::get_debit_pool(), 0);

		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(5, 2))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_noop!(
			CDPEngineModule::close_cdp_has_debit_by_dex(ALICE, SERP, 100),
			Error::<Runtime>::MustBeSafe
		);

		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));

		// max collateral amount limit swap
		assert_noop!(
			CDPEngineModule::close_cdp_has_debit_by_dex(ALICE, SERP, 5),
			cdp_treasury::Error::<Runtime>::CannotSwap,
		);

		assert_eq!(DEXModule::get_liquidity_pool(SERP, SETUSD), (100, 1000));
		assert_ok!(CDPEngineModule::close_cdp_has_debit_by_dex(ALICE, SERP, 6));
		System::assert_last_event(Event::CDPEngineModule(crate::Event::CloseCDPInDebitByDEX {
			collateral_type: SERP,
			owner: ALICE,
			sold_collateral_amount: 6,
			refund_collateral_amount: 94,
			debit_value: 50,
		}));

		assert_eq!(DEXModule::get_liquidity_pool(SERP, SETUSD), (106, 956));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 994);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 0);
		assert_eq!(CDPTreasuryModule::get_surplus_pool(), 50);
		assert_eq!(CDPTreasuryModule::get_debit_pool(), 50);
	});
}

#[test]
fn close_cdp_has_debit_by_swap_on_alternative_path() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(DEXModule::add_liquidity(
			Origin::signed(CAROL),
			SERP,
			SETM,
			100,
			1000,
			0
		));
		assert_ok!(DEXModule::add_liquidity(
			Origin::signed(CAROL),
			SETM,
			SETUSD,
			1000,
			1000,
			0
		));
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		assert_eq!(DEXModule::get_liquidity_pool(SERP, SETM), (100, 1000));
		assert_eq!(DEXModule::get_liquidity_pool(SETM, SETUSD), (1000, 1000));
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 100, 500));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 500);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);
		assert_eq!(CDPTreasuryModule::get_surplus_pool(), 0);
		assert_eq!(CDPTreasuryModule::get_debit_pool(), 0);

		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		assert_eq!(DEXModule::get_liquidity_pool(SERP, SETM), (100, 1000));
		assert_eq!(DEXModule::get_liquidity_pool(SETM, SETUSD), (1000, 1000));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 500);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);
		assert_eq!(CDPTreasuryModule::get_surplus_pool(), 0);
		assert_eq!(CDPTreasuryModule::get_debit_pool(), 0);
	});
}

#[test]
fn offchain_worker_works_cdp() {
	let (offchain, _offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
	let mut ext = ExtBuilder::default().build();
	ext.register_extension(OffchainWorkerExt::new(offchain.clone()));
	ext.register_extension(TransactionPoolExt::new(pool));
	ext.register_extension(OffchainDbExt::new(offchain));

	ext.execute_with(|| {
		// number of currencies allowed as collateral (cycles through all of them)
		let collateral_currencies_num = CollateralCurrencyIds::get().len() as u64;
		System::set_block_number(1);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		// offchain worker will not liquidate alice
		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 100, 500));
		assert_ok!(CDPEngineModule::adjust_position(&BOB, SERP, 100, 100));
		assert_eq!(Currencies::free_balance(SERP, &ALICE), 900);
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 50);
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 500);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);
		// jump 2 blocks at a time because code rotates through the different T::CollateralCurrencyIds
		run_to_block_offchain(System::block_number() + collateral_currencies_num);

		// checks that offchain worker tx pool is empty (therefore tx to liquidate alice is not present)
		assert!(pool_state.write().transactions.pop().is_none());
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 500);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 100);

		// changes alice into unsafe position
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		run_to_block_offchain(System::block_number() + collateral_currencies_num);

		// offchain worker will liquidate alice
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		if let MockCall::CDPEngineModule(crate::Call::liquidate(currency_call, who_call)) = tx.call {
			assert_ok!(CDPEngineModule::liquidate(Origin::none(), currency_call, who_call));
		}
		// empty offchain tx pool (Bob was not liquidated)
		assert!(pool_state.write().transactions.pop().is_none());
		// alice is liquidated but bob is not
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 0);
		assert_eq!(LoansModule::positions(SERP, BOB).debit, 100);
		assert_eq!(LoansModule::positions(SERP, BOB).collateral, 100);

		// emergency shutdown will settle Bobs debit position
		mock_shutdown();
		assert!(MockEmergencyShutdown::is_shutdown());
		run_to_block_offchain(System::block_number() + collateral_currencies_num);
		// offchain worker will settle bob's position
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		if let MockCall::CDPEngineModule(crate::Call::settle(currency_call, who_call)) = tx.call {
			assert_ok!(CDPEngineModule::settle(Origin::none(), currency_call, who_call));
		}
		// emergency shutdown settles bob's debit position
		assert_eq!(LoansModule::positions(SERP, BOB).debit, 0);
		assert_eq!(LoansModule::positions(SERP, BOB).collateral, 90);
	});
}

#[test]
fn offchain_worker_iteration_limit_works() {
	let (mut offchain, _offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
	let mut ext = ExtBuilder::default().build();
	ext.register_extension(OffchainWorkerExt::new(offchain.clone()));
	ext.register_extension(TransactionPoolExt::new(pool));
	ext.register_extension(OffchainDbExt::new(offchain.clone()));

	ext.execute_with(|| {
		System::set_block_number(1);
		// sets max iterations value to 1
		offchain.local_storage_set(StorageKind::PERSISTENT, OFFCHAIN_WORKER_MAX_ITERATIONS, &1u32.encode());
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));

		assert_ok!(CDPEngineModule::adjust_position(&ALICE, SERP, 100, 500));
		assert_ok!(CDPEngineModule::adjust_position(&BOB, SERP, 100, 500));
		// make both positions unsafe
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		run_to_block_offchain(2);
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		if let MockCall::CDPEngineModule(crate::Call::liquidate(currency_call, who_call)) = tx.call {
			assert_ok!(CDPEngineModule::liquidate(Origin::none(), currency_call, who_call));
		}
		// alice is liquidated but not bob, he will get liquidated next block due to iteration limit
		assert_eq!(LoansModule::positions(SERP, ALICE).debit, 0);
		assert_eq!(LoansModule::positions(SERP, ALICE).collateral, 0);
		// only one tx is submitted due to iteration limit
		assert!(pool_state.write().transactions.pop().is_none());

		// Iterator continues where it was from storage and now liquidates bob
		run_to_block_offchain(3);
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		if let MockCall::CDPEngineModule(crate::Call::liquidate(currency_call, who_call)) = tx.call {
			assert_ok!(CDPEngineModule::liquidate(Origin::none(), currency_call, who_call));
		}
		assert_eq!(LoansModule::positions(SERP, BOB).debit, 0);
		assert_eq!(LoansModule::positions(SERP, BOB).collateral, 0);
		assert!(pool_state.write().transactions.pop().is_none());
	});
}

#[test]
fn offchain_default_max_iterator_works() {
	let (mut offchain, _offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
	let mut ext = ExtBuilder::lots_of_accounts().build();
	ext.register_extension(OffchainWorkerExt::new(offchain.clone()));
	ext.register_extension(TransactionPoolExt::new(pool));
	ext.register_extension(OffchainDbExt::new(offchain.clone()));

	ext.execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 2))),
			Change::NewValue(Some(Rate::saturating_from_rational(2, 10))),
			Change::NewValue(Some(Ratio::saturating_from_rational(9, 5))),
			Change::NewValue(10000),
		));
		// checks that max iterations is stored as none
		assert!(offchain
			.local_storage_get(StorageKind::PERSISTENT, OFFCHAIN_WORKER_MAX_ITERATIONS)
			.is_none());

		for i in 0..1001 {
			let acount_id: AccountId = i;
			assert_ok!(CDPEngineModule::adjust_position(&acount_id, SERP, 10, 50));
		}

		// make all positions unsafe
		assert_ok!(CDPEngineModule::set_collateral_params(
			Origin::signed(1),
			SERP,
			Change::NewValue(Some(Ratio::saturating_from_rational(3, 1))),
			Change::NoChange,
			Change::NoChange,
			Change::NoChange,
		));
		run_to_block_offchain(2);
		// should only run 1000 iterations stopping due to DEFAULT_MAX_ITERATIONS
		assert_eq!(pool_state.write().transactions.len(), 1000);
		// should only now run 1 iteration to finish off where it ended last block
		run_to_block_offchain(3);
		assert_eq!(pool_state.write().transactions.len(), 1001);
	});
}
