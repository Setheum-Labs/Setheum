// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of Setheum.

// Copyright (C) 2019-Present Setheum Labs.
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

//! Unit tests for the cdp treasury module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{RuntimeEvent, *};
use module_support::SwapError;
use sp_runtime::traits::BadOrigin;

#[test]
fn surplus_pool_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 0);
		assert_ok!(Currencies::deposit(
			GetUSSDCurrencyId::get(),
			&SlickUsdEcdpTreasuryModule::account_id(),
			500
		));
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 500);
	});
}

#[test]
fn total_collaterals_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals(BTC), 0);
		assert_ok!(Currencies::deposit(BTC, &SlickUsdEcdpTreasuryModule::account_id(), 10));
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals(BTC), 10);
	});
}

#[test]
fn on_system_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 0);
		assert_ok!(SlickUsdEcdpTreasuryModule::on_system_debit(1000));
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 1000);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::on_system_debit(Balance::max_value()),
			ArithmeticError::Overflow,
		);
	});
}

#[test]
fn on_system_surplus_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 0);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 0);
		assert_ok!(SlickUsdEcdpTreasuryModule::on_system_surplus(1000));
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 1000);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 1000);
	});
}

#[test]
fn offset_surplus_and_debit_on_finalize_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 0);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 0);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 0);
		assert_ok!(SlickUsdEcdpTreasuryModule::on_system_surplus(1000));
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 1000);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 1000);
		SlickUsdEcdpTreasuryModule::on_finalize(1);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 1000);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 1000);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 0);
		assert_ok!(SlickUsdEcdpTreasuryModule::on_system_debit(300));
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 300);
		SlickUsdEcdpTreasuryModule::on_finalize(2);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 700);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 700);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 0);
		assert_ok!(SlickUsdEcdpTreasuryModule::on_system_debit(800));
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 800);
		SlickUsdEcdpTreasuryModule::on_finalize(3);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 0);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 0);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 100);
	});
}

#[test]
fn issue_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USSD, &ALICE), 1000);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 0);

		assert_ok!(SlickUsdEcdpTreasuryModule::issue_debit(&ALICE, 1000, true));
		assert_eq!(Currencies::free_balance(USSD, &ALICE), 2000);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 0);

		assert_ok!(SlickUsdEcdpTreasuryModule::issue_debit(&ALICE, 1000, false));
		assert_eq!(Currencies::free_balance(USSD, &ALICE), 3000);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 1000);
	});
}

#[test]
fn burn_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USSD, &ALICE), 1000);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 0);
		assert_ok!(SlickUsdEcdpTreasuryModule::burn_debit(&ALICE, 300));
		assert_eq!(Currencies::free_balance(USSD, &ALICE), 700);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 0);
	});
}

#[test]
fn deposit_surplus_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USSD, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 0);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 0);
		assert_ok!(SlickUsdEcdpTreasuryModule::deposit_surplus(&ALICE, 300));
		assert_eq!(Currencies::free_balance(USSD, &ALICE), 700);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 300);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 300);
	});
}

#[test]
fn withdraw_surplus_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SlickUsdEcdpTreasuryModule::deposit_surplus(&ALICE, 300));
		assert_eq!(Currencies::free_balance(USSD, &ALICE), 700);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 300);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 300);

		assert_ok!(SlickUsdEcdpTreasuryModule::withdraw_surplus(&ALICE, 200));
		assert_eq!(Currencies::free_balance(USSD, &ALICE), 900);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 100);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 100);
	});
}

#[test]
fn deposit_collateral_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals(BTC), 0);
		assert_eq!(Currencies::free_balance(BTC, &SlickUsdEcdpTreasuryModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 1000);
		assert!(!SlickUsdEcdpTreasuryModule::deposit_collateral(&ALICE, BTC, 10000).is_ok());
		assert_ok!(SlickUsdEcdpTreasuryModule::deposit_collateral(&ALICE, BTC, 500));
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals(BTC), 500);
		assert_eq!(Currencies::free_balance(BTC, &SlickUsdEcdpTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(BTC, &ALICE), 500);
	});
}

#[test]
fn withdraw_collateral_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SlickUsdEcdpTreasuryModule::deposit_collateral(&ALICE, BTC, 500));
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals(BTC), 500);
		assert_eq!(Currencies::free_balance(BTC, &SlickUsdEcdpTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(BTC, &BOB), 1000);
		assert!(!SlickUsdEcdpTreasuryModule::withdraw_collateral(&BOB, BTC, 501).is_ok());
		assert_ok!(SlickUsdEcdpTreasuryModule::withdraw_collateral(&BOB, BTC, 400));
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals(BTC), 100);
		assert_eq!(Currencies::free_balance(BTC, &SlickUsdEcdpTreasuryModule::account_id()), 100);
		assert_eq!(Currencies::free_balance(BTC, &BOB), 1400);
	});
}

#[test]
fn get_total_collaterals_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SlickUsdEcdpTreasuryModule::deposit_collateral(&ALICE, BTC, 500));
		assert_eq!(SlickUsdEcdpTreasuryModule::get_total_collaterals(BTC), 500);
	});
}

#[test]
fn get_debit_proportion_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			SlickUsdEcdpTreasuryModule::get_debit_proportion(100),
			Ratio::saturating_from_rational(100, Currencies::total_issuance(USSD))
		);
	});
}

#[test]
fn swap_collateral_to_ussd_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SlickUsdEcdpTreasuryModule::deposit_collateral(&BOB, BTC, 200));
		assert_ok!(SlickUsdEcdpTreasuryModule::deposit_collateral(&CHARLIE, EDF, 1000));
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals_not_in_auction(BTC), 200);
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals_not_in_auction(EDF), 1000);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 0);
		assert_ok!(DEXModule::add_liquidity(
			RuntimeOrigin::signed(BOB),
			EDF,
			USSD,
			1000,
			1000,
			0,
			false
		));

		assert_noop!(
			SlickUsdEcdpTreasuryModule::swap_collateral_to_ussd(BTC, SwapLimit::ExactTarget(201, 200), false),
			Error::<Runtime>::CollateralNotEnough,
		);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::swap_collateral_to_ussd(EDF, SwapLimit::ExactSupply(1001, 0), false),
			Error::<Runtime>::CollateralNotEnough,
		);

		assert_noop!(
			SlickUsdEcdpTreasuryModule::swap_collateral_to_ussd(BTC, SwapLimit::ExactTarget(200, 399), false),
			SwapError::CannotSwap
		);
		assert_ok!(DEXModule::add_liquidity(
			RuntimeOrigin::signed(ALICE),
			BTC,
			EDF,
			100,
			1000,
			0,
			false
		));

		assert_eq!(
			SlickUsdEcdpTreasuryModule::swap_collateral_to_ussd(BTC, SwapLimit::ExactTarget(200, 399), false).unwrap(),
			(198, 399)
		);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 399);
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals_not_in_auction(BTC), 2);

		assert_noop!(
			SlickUsdEcdpTreasuryModule::swap_collateral_to_ussd(EDF, SwapLimit::ExactSupply(1000, 1000), false),
			SwapError::CannotSwap
		);

		assert_eq!(
			SlickUsdEcdpTreasuryModule::swap_collateral_to_ussd(EDF, SwapLimit::ExactSupply(1000, 0), false).unwrap(),
			(1000, 225)
		);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 624);
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals_not_in_auction(EDF), 0);
	});
}

#[test]
fn create_collateral_auctions_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Currencies::deposit(BTC, &SlickUsdEcdpTreasuryModule::account_id(), 10000));
		assert_eq!(SlickUsdEcdpTreasuryModule::expected_collateral_auction_size(BTC), 0);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::create_collateral_auctions(BTC, 10001, 1000, ALICE, true),
			Error::<Runtime>::CollateralNotEnough,
		);

		// without collateral auction maximum size
		assert_ok!(SlickUsdEcdpTreasuryModule::create_collateral_auctions(
			BTC, 1000, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 1);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 1000);

		// set collateral auction maximum size
		assert_ok!(SlickUsdEcdpTreasuryModule::set_expected_collateral_auction_size(
			RuntimeOrigin::signed(1),
			BTC,
			300
		));

		// amount < collateral auction maximum size
		// auction + 1
		assert_ok!(SlickUsdEcdpTreasuryModule::create_collateral_auctions(
			BTC, 200, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 2);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 1200);

		// not exceed lots count cap
		// auction + 4
		assert_ok!(SlickUsdEcdpTreasuryModule::create_collateral_auctions(
			BTC, 1000, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 6);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 2200);

		// exceed lots count cap
		// auction + 5
		assert_ok!(SlickUsdEcdpTreasuryModule::create_collateral_auctions(
			BTC, 2000, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 11);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 4200);
	});
}

#[test]
fn remove_liquidity_for_lp_collateral_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEXModule::add_liquidity(
			RuntimeOrigin::signed(BOB),
			USSD,
			EDF,
			1000,
			100,
			0,
			false
		));
		assert_ok!(SlickUsdEcdpTreasuryModule::deposit_collateral(&BOB, LP_USSD_EDF, 200));
		assert_eq!(Currencies::total_issuance(LP_USSD_EDF), 2000);
		assert_eq!(DEXModule::get_liquidity_pool(USSD, EDF), (1000, 100));
		assert_eq!(
			Currencies::free_balance(LP_USSD_EDF, &SlickUsdEcdpTreasuryModule::account_id()),
			200
		);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(EDF, &SlickUsdEcdpTreasuryModule::account_id()), 0);

		assert_noop!(
			SlickUsdEcdpTreasuryModule::remove_liquidity_for_lp_collateral(EDF, 200),
			Error::<Runtime>::NotDexShare
		);

		assert_eq!(
			SlickUsdEcdpTreasuryModule::remove_liquidity_for_lp_collateral(LP_USSD_EDF, 120),
			Ok((60, 6))
		);
		assert_eq!(Currencies::total_issuance(LP_USSD_EDF), 1880);
		assert_eq!(DEXModule::get_liquidity_pool(USSD, EDF), (940, 94));
		assert_eq!(
			Currencies::free_balance(LP_USSD_EDF, &SlickUsdEcdpTreasuryModule::account_id()),
			80
		);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 60);
		assert_eq!(Currencies::free_balance(EDF, &SlickUsdEcdpTreasuryModule::account_id()), 6);
	});
}

#[test]
fn set_expected_collateral_auction_size_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_eq!(SlickUsdEcdpTreasuryModule::expected_collateral_auction_size(BTC), 0);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::set_expected_collateral_auction_size(RuntimeOrigin::signed(5), BTC, 200),
			BadOrigin
		);
		assert_ok!(SlickUsdEcdpTreasuryModule::set_expected_collateral_auction_size(
			RuntimeOrigin::signed(1),
			BTC,
			200
		));
		System::assert_last_event(RuntimeEvent::SlickUsdEcdpTreasuryModule(
			crate::Event::ExpectedCollateralAuctionSizeUpdated {
				collateral_type: BTC,
				new_size: 200,
			},
		));
	});
}

#[test]
fn extract_surplus_to_treasury_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SlickUsdEcdpTreasuryModule::on_system_surplus(1000));
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 1000);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 1000);
		assert_eq!(Currencies::free_balance(USSD, &TreasuryAccount::get()), 0);

		assert_noop!(
			SlickUsdEcdpTreasuryModule::extract_surplus_to_treasury(RuntimeOrigin::signed(5), 200),
			BadOrigin
		);
		assert_ok!(SlickUsdEcdpTreasuryModule::extract_surplus_to_treasury(
			RuntimeOrigin::signed(1),
			200
		));
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 800);
		assert_eq!(Currencies::free_balance(USSD, &SlickUsdEcdpTreasuryModule::account_id()), 800);
		assert_eq!(Currencies::free_balance(USSD, &TreasuryAccount::get()), 200);
	});
}

#[test]
fn auction_collateral_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Currencies::deposit(BTC, &SlickUsdEcdpTreasuryModule::account_id(), 10000));
		assert_eq!(SlickUsdEcdpTreasuryModule::expected_collateral_auction_size(BTC), 0);
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals(BTC), 10000);
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals_not_in_auction(BTC), 10000);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::auction_collateral(RuntimeOrigin::signed(5), BTC, 10000, 1000, false),
			BadOrigin,
		);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::auction_collateral(RuntimeOrigin::signed(1), BTC, 10001, 1000, false),
			Error::<Runtime>::CollateralNotEnough,
		);

		assert_ok!(SlickUsdEcdpTreasuryModule::auction_collateral(
			RuntimeOrigin::signed(1),
			BTC,
			1000,
			1000,
			false
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 1);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 1000);

		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals(BTC), 10000);
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals_not_in_auction(BTC), 9000);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::auction_collateral(RuntimeOrigin::signed(1), BTC, 9001, 1000, false),
			Error::<Runtime>::CollateralNotEnough,
		);
	});
}

#[test]
fn exchange_collateral_to_ussd_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEXModule::add_liquidity(
			RuntimeOrigin::signed(BOB),
			BTC,
			USSD,
			200,
			1000,
			0,
			false
		));

		assert_ok!(Currencies::deposit(BTC, &SlickUsdEcdpTreasuryModule::account_id(), 1000));
		assert_ok!(SlickUsdEcdpTreasuryModule::auction_collateral(
			RuntimeOrigin::signed(1),
			BTC,
			800,
			1000,
			false
		));
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals(BTC), 1000);
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals_not_in_auction(BTC), 200);
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 0);

		assert_noop!(
			SlickUsdEcdpTreasuryModule::exchange_collateral_to_ussd(
				RuntimeOrigin::signed(5),
				BTC,
				SwapLimit::ExactTarget(200, 200)
			),
			BadOrigin,
		);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::exchange_collateral_to_ussd(
				RuntimeOrigin::signed(1),
				BTC,
				SwapLimit::ExactTarget(201, 200)
			),
			Error::<Runtime>::CollateralNotEnough,
		);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::exchange_collateral_to_ussd(
				RuntimeOrigin::signed(1),
				BTC,
				SwapLimit::ExactSupply(201, 0)
			),
			Error::<Runtime>::CollateralNotEnough,
		);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::exchange_collateral_to_ussd(
				RuntimeOrigin::signed(1),
				BTC,
				SwapLimit::ExactTarget(200, 1000)
			),
			SwapError::CannotSwap
		);

		assert_ok!(SlickUsdEcdpTreasuryModule::exchange_collateral_to_ussd(
			RuntimeOrigin::signed(1),
			BTC,
			SwapLimit::ExactTarget(200, 399)
		));
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 399);
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals(BTC), 867);
		assert_eq!(SlickUsdEcdpTreasuryModule::total_collaterals_not_in_auction(BTC), 67);
	});
}

#[test]
fn set_debit_offset_buffer_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_offset_buffer(), 0);
		assert_noop!(
			SlickUsdEcdpTreasuryModule::set_debit_offset_buffer(RuntimeOrigin::signed(5), 200),
			BadOrigin
		);
		assert_ok!(SlickUsdEcdpTreasuryModule::set_debit_offset_buffer(
			RuntimeOrigin::signed(1),
			200
		));
		System::assert_last_event(RuntimeEvent::SlickUsdEcdpTreasuryModule(
			crate::Event::DebitOffsetBufferUpdated { amount: 200 },
		));
	});
}

#[test]
fn offset_surplus_and_debit_limited_by_debit_offset_buffer() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SlickUsdEcdpTreasuryModule::on_system_surplus(1000));
		assert_ok!(SlickUsdEcdpTreasuryModule::on_system_debit(2000));
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 1000);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 2000);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_offset_buffer(), 0);

		// offset all debit pool when surplus is enough
		SlickUsdEcdpTreasuryModule::offset_surplus_and_debit();
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 0);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 1000);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_offset_buffer(), 0);

		assert_ok!(SlickUsdEcdpTreasuryModule::set_debit_offset_buffer(
			RuntimeOrigin::signed(1),
			100
		));
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_offset_buffer(), 100);
		assert_ok!(SlickUsdEcdpTreasuryModule::on_system_surplus(2000));
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 2000);

		// keep the buffer for debit pool when surplus is enough
		SlickUsdEcdpTreasuryModule::offset_surplus_and_debit();
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 1100);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 100);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_offset_buffer(), 100);

		assert_ok!(SlickUsdEcdpTreasuryModule::set_debit_offset_buffer(
			RuntimeOrigin::signed(1),
			200
		));
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_offset_buffer(), 200);
		assert_ok!(SlickUsdEcdpTreasuryModule::on_system_debit(1400));
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 1500);

		SlickUsdEcdpTreasuryModule::offset_surplus_and_debit();
		assert_eq!(SlickUsdEcdpTreasuryModule::surplus_pool(), 0);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_pool(), 400);
		assert_eq!(SlickUsdEcdpTreasuryModule::debit_offset_buffer(), 200);
	});
}
