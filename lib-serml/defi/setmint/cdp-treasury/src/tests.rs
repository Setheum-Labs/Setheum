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

//! Unit tests for the cdp treasury module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn surplus_pool_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
		assert_ok!(Currencies::deposit(
			GetSetUSDId::get(),
			&CDPTreasuryModule::account_id(),
			500
		));
		assert_eq!(CDPTreasuryModule::surplus_pool(), 500);
	});
}

#[test]
fn total_collaterals_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(CDPTreasuryModule::total_collaterals(ETH), 0);
		assert_ok!(Currencies::deposit(ETH, &CDPTreasuryModule::account_id(), 10));
		assert_eq!(CDPTreasuryModule::total_collaterals(ETH), 10);
	});
}

#[test]
fn on_system_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
		assert_ok!(CDPTreasuryModule::on_system_debit(1000));
		assert_eq!(CDPTreasuryModule::debit_pool(), 1000);
		assert_noop!(
			CDPTreasuryModule::on_system_debit(Balance::max_value()),
			ArithmeticError::Overflow,
		);
	});
}

#[test]
fn on_system_surplus_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 0);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
		assert_ok!(CDPTreasuryModule::on_system_surplus(1000));
		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 1000);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 1000);
	});
}

#[test]
fn offset_surplus_and_debit_on_finalize_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 0);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
		assert_ok!(CDPTreasuryModule::on_system_surplus(1000));
		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 1000);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 1000);
		CDPTreasuryModule::on_finalize(1);
		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 1000);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 1000);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
		assert_ok!(CDPTreasuryModule::on_system_debit(300));
		assert_eq!(CDPTreasuryModule::debit_pool(), 300);
		CDPTreasuryModule::on_finalize(2);
		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 700);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 700);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
		assert_ok!(CDPTreasuryModule::on_system_debit(800));
		assert_eq!(CDPTreasuryModule::debit_pool(), 800);
		CDPTreasuryModule::on_finalize(3);
		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 0);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
		assert_eq!(CDPTreasuryModule::debit_pool(), 100);
	});
}

#[test]
fn issue_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDI, &ALICE), 1000);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);

		assert_ok!(CDPTreasuryModule::issue_debit(&ALICE, 1000, true));
		assert_eq!(Currencies::free_balance(USDI, &ALICE), 2000);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);

		assert_ok!(CDPTreasuryModule::issue_debit(&ALICE, 1000, false));
		assert_eq!(Currencies::free_balance(USDI, &ALICE), 3000);
		assert_eq!(CDPTreasuryModule::debit_pool(), 1000);
	});
}

#[test]
fn burn_debit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDI, &ALICE), 1000);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
		assert_ok!(CDPTreasuryModule::burn_debit(&ALICE, 300));
		assert_eq!(Currencies::free_balance(USDI, &ALICE), 700);
		assert_eq!(CDPTreasuryModule::debit_pool(), 0);
	});
}

#[test]
fn deposit_surplus_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(USDI, &ALICE), 1000);
		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 0);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
		assert_ok!(CDPTreasuryModule::deposit_surplus(&ALICE, 300));
		assert_eq!(Currencies::free_balance(USDI, &ALICE), 700);
		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 300);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 300);
	});
}

#[test]
fn deposit_collateral_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(CDPTreasuryModule::total_collaterals(ETH), 0);
		assert_eq!(Currencies::free_balance(ETH, &CDPTreasuryModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(ETH, &ALICE), 1000);
		assert!(!CDPTreasuryModule::deposit_collateral(&ALICE, ETH, 10000).is_ok());
		assert_ok!(CDPTreasuryModule::deposit_collateral(&ALICE, ETH, 500));
		assert_eq!(CDPTreasuryModule::total_collaterals(ETH), 500);
		assert_eq!(Currencies::free_balance(ETH, &CDPTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(ETH, &ALICE), 500);
	});
}

#[test]
fn withdraw_collateral_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPTreasuryModule::deposit_collateral(&ALICE, ETH, 500));
		assert_eq!(CDPTreasuryModule::total_collaterals(ETH), 500);
		assert_eq!(Currencies::free_balance(ETH, &CDPTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(ETH, &BOB), 1000);
		assert!(!CDPTreasuryModule::withdraw_collateral(&BOB, ETH, 501).is_ok());
		assert_ok!(CDPTreasuryModule::withdraw_collateral(&BOB, ETH, 400));
		assert_eq!(CDPTreasuryModule::total_collaterals(ETH), 100);
		assert_eq!(Currencies::free_balance(ETH, &CDPTreasuryModule::account_id()), 100);
		assert_eq!(Currencies::free_balance(ETH, &BOB), 1400);
	});
}

#[test]
fn get_total_collaterals_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPTreasuryModule::deposit_collateral(&ALICE, ETH, 500));
		assert_eq!(CDPTreasuryModule::get_total_collaterals(ETH), 500);
	});
}

#[test]
fn get_debit_proportion_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			CDPTreasuryModule::get_debit_proportion(100),
			Ratio::saturating_from_rational(100, Currencies::total_issuance(USDI))
		);
	});
}

#[test]
fn swap_collateral_to_stable_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CDPTreasuryModule::deposit_collateral(&BOB, ETH, 200));
		assert_ok!(CDPTreasuryModule::deposit_collateral(&CHARLIE, WBTC, 1000));
		assert_eq!(CDPTreasuryModule::total_collaterals_not_in_auction(ETH), 200);
		assert_eq!(CDPTreasuryModule::total_collaterals_not_in_auction(WBTC), 1000);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);
		assert_ok!(DEXModule::add_liquidity(
			Origin::signed(BOB),
			WBTC,
			USDI,
			1000,
			1000,
			0,
		));

		assert_noop!(
			CDPTreasuryModule::swap_collateral_to_stable(ETH, SwapLimit::ExactTarget(201, 200), false),
			Error::<Runtime>::CollateralNotEnough,
		);
		assert_noop!(
			CDPTreasuryModule::swap_collateral_to_stable(WBTC, SwapLimit::ExactSupply(1001, 0), false),
			Error::<Runtime>::CollateralNotEnough,
		);

		assert_noop!(
			CDPTreasuryModule::swap_collateral_to_stable(ETH, SwapLimit::ExactTarget(200, 399), false),
			Error::<Runtime>::CannotSwap
		);
		assert_ok!(DEXModule::add_liquidity(
			Origin::signed(ALICE),
			ETH,
			WBTC,
			100,
			1000,
			0,
		));

		assert_eq!(
			CDPTreasuryModule::swap_collateral_to_stable(ETH, SwapLimit::ExactTarget(200, 399), false).unwrap(),
			(198, 399)
		);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 399);
		assert_eq!(CDPTreasuryModule::total_collaterals_not_in_auction(ETH), 2);

		assert_noop!(
			CDPTreasuryModule::swap_collateral_to_stable(WBTC, SwapLimit::ExactSupply(1000, 1000), false),
			Error::<Runtime>::CannotSwap,
		);

		assert_eq!(
			CDPTreasuryModule::swap_collateral_to_stable(WBTC, SwapLimit::ExactSupply(1000, 0), false).unwrap(),
			(1000, 225)
		);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 624);
		assert_eq!(CDPTreasuryModule::total_collaterals_not_in_auction(WBTC), 0);
	});
}

#[test]
fn create_collateral_auctions_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Currencies::deposit(ETH, &CDPTreasuryModule::account_id(), 10000));
		assert_eq!(CDPTreasuryModule::expected_collateral_auction_size(ETH), 0);
		assert_noop!(
			CDPTreasuryModule::create_collateral_auctions(ETH, 10001, 1000, ALICE, true),
			Error::<Runtime>::CollateralNotEnough,
		);

		// without collateral auction maximum size
		assert_ok!(CDPTreasuryModule::create_collateral_auctions(
			ETH, 1000, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 1);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 1000);

		// set collateral auction maximum size
		assert_ok!(CDPTreasuryModule::set_expected_collateral_auction_size(
			Origin::signed(1),
			ETH,
			300
		));

		// amount < collateral auction maximum size
		// auction + 1
		assert_ok!(CDPTreasuryModule::create_collateral_auctions(
			ETH, 200, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 2);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 1200);

		// not exceed lots count cap
		// auction + 4
		assert_ok!(CDPTreasuryModule::create_collateral_auctions(
			ETH, 1000, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 6);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 2200);

		// exceed lots count cap
		// auction + 5
		assert_ok!(CDPTreasuryModule::create_collateral_auctions(
			ETH, 2000, 1000, ALICE, true
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 11);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 4200);
	});
}

// #[test]
// fn remove_liquidity_for_lp_collateral_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_ok!(DEXModule::add_liquidity(
// 			Origin::signed(BOB),
// 			USDI,
// 			WBTC,
// 			1000,
// 			100,
// 			0,
// 		));
// 		assert_ok!(CDPTreasuryModule::deposit_collateral(&BOB, LP_USDI_WBTC, 200));
// 		assert_eq!(Currencies::total_issuance(LP_USDI_WBTC), 2000);
// 		assert_eq!(DEXModule::get_liquidity_pool(USDI, WBTC), (1000, 100));
// 		assert_eq!(
// 			Currencies::free_balance(LP_USDI_WBTC, &CDPTreasuryModule::account_id()),
// 			200
// 		);
// 		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 0);
// 		assert_eq!(Currencies::free_balance(WBTC, &CDPTreasuryModule::account_id()), 0);

// 		assert_noop!(
// 			CDPTreasuryModule::remove_liquidity_for_lp_collateral(WBTC, 200),
// 			Error::<Runtime>::NotDexShare
// 		);

// 		assert_eq!(
// 			CDPTreasuryModule::remove_liquidity_for_lp_collateral(LP_USDI_WBTC, 120),
// 			Ok((60, 6))
// 		);
// 		assert_eq!(Currencies::total_issuance(LP_USDI_WBTC), 1880);
// 		assert_eq!(DEXModule::get_liquidity_pool(USDI, WBTC), (940, 94));
// 		assert_eq!(
// 			Currencies::free_balance(LP_USDI_WBTC, &CDPTreasuryModule::account_id()),
// 			80
// 		);
// 		assert_eq!(Currencies::free_balance(USDI, &CDPTreasuryModule::account_id()), 60);
// 		assert_eq!(Currencies::free_balance(WBTC, &CDPTreasuryModule::account_id()), 6);
// 	});
// }

#[test]
fn set_expected_collateral_auction_size_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_eq!(CDPTreasuryModule::expected_collateral_auction_size(ETH), 0);
		assert_noop!(
			CDPTreasuryModule::set_expected_collateral_auction_size(Origin::signed(5), ETH, 200),
			BadOrigin
		);
		assert_ok!(CDPTreasuryModule::set_expected_collateral_auction_size(
			Origin::signed(1),
			ETH,
			200
		));
		System::assert_last_event(Event::CDPTreasuryModule(
			crate::Event::ExpectedCollateralAuctionSizeUpdated {
				collateral_type: ETH,
				new_size: 200,
			},
		));
	});
}

#[test]
fn auction_collateral_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Currencies::deposit(ETH, &CDPTreasuryModule::account_id(), 10000));
		assert_eq!(CDPTreasuryModule::expected_collateral_auction_size(ETH), 0);
		assert_eq!(CDPTreasuryModule::total_collaterals(ETH), 10000);
		assert_eq!(CDPTreasuryModule::total_collaterals_not_in_auction(ETH), 10000);
		assert_noop!(
			CDPTreasuryModule::auction_collateral(Origin::signed(5), ETH, 10000, 1000, false),
			BadOrigin,
		);
		assert_noop!(
			CDPTreasuryModule::auction_collateral(Origin::signed(1), ETH, 10001, 1000, false),
			Error::<Runtime>::CollateralNotEnough,
		);

		assert_ok!(CDPTreasuryModule::auction_collateral(
			Origin::signed(1),
			ETH,
			1000,
			1000,
			false
		));
		assert_eq!(TOTAL_COLLATERAL_AUCTION.with(|v| *v.borrow_mut()), 1);
		assert_eq!(TOTAL_COLLATERAL_IN_AUCTION.with(|v| *v.borrow_mut()), 1000);

		assert_eq!(CDPTreasuryModule::total_collaterals(ETH), 10000);
		assert_eq!(CDPTreasuryModule::total_collaterals_not_in_auction(ETH), 9000);
		assert_noop!(
			CDPTreasuryModule::auction_collateral(Origin::signed(1), ETH, 9001, 1000, false),
			Error::<Runtime>::CollateralNotEnough,
		);
	});
}

#[test]
fn exchange_collateral_to_stable_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(DEXModule::add_liquidity(
			Origin::signed(BOB),
			ETH,
			USDI,
			200,
			1000,
			0,
		));

		assert_ok!(Currencies::deposit(ETH, &CDPTreasuryModule::account_id(), 1000));
		assert_ok!(CDPTreasuryModule::auction_collateral(
			Origin::signed(1),
			ETH,
			800,
			1000,
			false
		));
		assert_eq!(CDPTreasuryModule::total_collaterals(ETH), 1000);
		assert_eq!(CDPTreasuryModule::total_collaterals_not_in_auction(ETH), 200);
		assert_eq!(CDPTreasuryModule::surplus_pool(), 0);

		assert_noop!(
			CDPTreasuryModule::exchange_collateral_to_stable(Origin::signed(5), ETH, SwapLimit::ExactTarget(200, 200)),
			BadOrigin,
		);
		assert_noop!(
			CDPTreasuryModule::exchange_collateral_to_stable(Origin::signed(1), ETH, SwapLimit::ExactTarget(201, 200)),
			Error::<Runtime>::CollateralNotEnough,
		);
		assert_noop!(
			CDPTreasuryModule::exchange_collateral_to_stable(Origin::signed(1), ETH, SwapLimit::ExactSupply(201, 0)),
			Error::<Runtime>::CollateralNotEnough,
		);
		assert_noop!(
			CDPTreasuryModule::exchange_collateral_to_stable(Origin::signed(1), ETH, SwapLimit::ExactTarget(200, 1000)),
			Error::<Runtime>::CannotSwap
		);

		assert_ok!(CDPTreasuryModule::exchange_collateral_to_stable(
			Origin::signed(1),
			ETH,
			SwapLimit::ExactTarget(200, 399)
		));
		assert_eq!(CDPTreasuryModule::surplus_pool(), 399);
		assert_eq!(CDPTreasuryModule::total_collaterals(ETH), 867);
		assert_eq!(CDPTreasuryModule::total_collaterals_not_in_auction(ETH), 67);
	});
}
