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

//! Unit tests for the dex module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{
	DexModule, Event, ExtBuilder, ListingOrigin, Origin, Runtime, System, Tokens, ALICE, SETUSD, SETUSD_DNAR_PAIR,
	SETUSD_SETHEUM_PAIR, BOB, DNAR, SETHEUM,
};
use orml_traits::MultiReservableCurrency;
use sp_runtime::traits::BadOrigin;

#[test]
fn enable_new_trading_pair_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			DexModule::enable_trading_pair(Origin::signed(ALICE), SETUSD, DNAR),
			BadOrigin
		);

		assert_eq!(
			DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);
		assert_ok!(DexModule::enable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DNAR
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);

		assert_noop!(
			DexModule::enable_trading_pair(Origin::signed(ListingOrigin::get()), DNAR, SETUSD),
			Error::<Runtime>::MustBeNotEnabled
		);
	});
}

#[test]
fn list_new_trading_pair_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			DexModule::list_trading_pair(
				Origin::signed(ALICE),
				SETUSD,
				DNAR,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			BadOrigin
		);

		assert_eq!(
			DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);
		assert_ok!(DexModule::list_trading_pair(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DNAR,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);

		assert_noop!(
			DexModule::list_trading_pair(
				Origin::signed(ListingOrigin::get()),
				SETUSD,
				DNAR,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			Error::<Runtime>::MustBeNotEnabled
		);
	});
}

#[test]
fn disable_enabled_trading_pair_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(DexModule::enable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DNAR
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);

		assert_noop!(
			DexModule::disable_trading_pair(Origin::signed(ALICE), SETUSD, DNAR),
			BadOrigin
		);

		assert_ok!(DexModule::disable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DNAR
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);

		Event::dex(crate::Event::DisableTradingPair(SETUSD_DNAR_PAIR));

		assert_noop!(
			DexModule::disable_trading_pair(Origin::signed(ListingOrigin::get()), SETUSD, DNAR),
			Error::<Runtime>::NotEnabledTradingPair
		);
	});
}

#[test]
fn disable_provisioning_trading_pair_work() {
	ExtBuilder::default()
		.initialize_listing_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				DNAR,
				5_000_000_000_000u128,
				0,
				0,
			));
			assert_ok!(DexModule::add_liquidity(
				Origin::signed(BOB),
				SETUSD,
				DNAR,
				5_000_000_000_000u128,
				1_000_000_000_000u128,
				0,
			));

			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 999_995_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 999_995_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_999_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				10_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(DNAR, &DexModule::account_id()),
				1_000_000_000_000u128
			);
			assert_eq!(
				DexModule::provisioning_pool(SETUSD_DNAR_PAIR, ALICE),
				(0, 0)
			);
			assert_eq!(
				DexModule::provisioning_pool(SETUSD_DNAR_PAIR, BOB),
				(0, 0)
			);
			assert_eq!(
				DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (0, 0),
					not_before: 10,
				})
			);
			let alice_ref_count_0 = System::consumers(&ALICE);
			let bob_ref_count_0 = System::consumers(&BOB);

			assert_ok!(DexModule::disable_trading_pair(
				Origin::signed(ListingOrigin::get()),
				SETUSD,
				DNAR
			));
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 0);
			assert_eq!(DexModule::provisioning_pool(SETUSD_DNAR_PAIR, ALICE), (0, 0));
			assert_eq!(DexModule::provisioning_pool(SETUSD_DNAR_PAIR, BOB), (0, 0));
			assert_eq!(
				DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (0, 0),
					not_before: 10,
				})
			);
			assert_eq!(System::consumers(&ALICE), alice_ref_count_0 - 1);
			assert_eq!(System::consumers(&BOB), bob_ref_count_0 - 1);
		});
}

#[test]
fn add_provision_work() {
	ExtBuilder::default()
		.initialize_listing_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_noop!(
				DexModule::add_liquidity(
					Origin::signed(ALICE),
					SETUSD,
					DNAR,
					4_000_000_000u128,
					900_000_000u128,
					0,
				),
				Error::<Runtime>::InvalidContributionIncrement
			);

			// alice add provision
			assert_eq!(
				DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (0, 0),
					not_before: 10,
				})
			);
			assert_eq!(DexModule::provisioning_pool(SETUSD_DNAR_PAIR, ALICE), (0, 0));
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 0);
			let alice_ref_count_0 = System::consumers(&ALICE);

			assert_eq!(
				DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (0, 0),
					not_before: 10,
				})
			);
			assert_eq!(
				DexModule::provisioning_pool(SETUSD_DNAR_PAIR, ALICE),
				(0, 0)
			);
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 1000000000000000000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				0
			);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 0);
			let alice_ref_count_1 = System::consumers(&ALICE);
			assert_eq!(alice_ref_count_1, alice_ref_count_0 + 0);

			Event::dex(crate::Event::AddProvision(ALICE, SETUSD, 5_000_000_000_000u128, DNAR, 0));

			// bob add provision
			assert_eq!(DexModule::provisioning_pool(SETUSD_DNAR_PAIR, BOB), (0, 0));
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000u128);
			let bob_ref_count_0 = System::consumers(&BOB);

			assert_ok!(DexModule::add_liquidity(
				Origin::signed(BOB),
				DNAR,
				SETUSD,
				1_000_000_000_000_000u128,
				0,
				0,
			));
			assert_eq!(
				DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (0, 0),
					not_before: 10,
				})
			);
			assert_eq!(
				DexModule::provisioning_pool(SETUSD_DNAR_PAIR, BOB),
				(0, 0)
			);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_000_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				0
			);
			assert_eq!(
				Tokens::free_balance(DNAR, &DexModule::account_id()),
				1_000_000_000_000_000u128
			);
			let bob_ref_count_1 = System::consumers(&BOB);
			assert_eq!(bob_ref_count_1, bob_ref_count_0 + 1);

			Event::dex(crate::Event::AddProvision(BOB, SETUSD, 0, DNAR, 1_000_000_000_000_000u128));

			// alice add provision again and trigger trading pair convert to Enabled from
			// Provisioning
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 1000000000000000000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(
				Tokens::total_issuance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap()),
				0
			);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);

			System::set_block_number(10);
			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				DNAR,
				995_000_000_000_000u128,
				1_000_000_000_000_000u128,
				0,
			));
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 999_005_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_000_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				995000000000000u128
			);
			assert_eq!(
				Tokens::free_balance(DNAR, &DexModule::account_id()),
				2_000_000_000_000_000u128
			);
			assert_eq!(
				Tokens::total_issuance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap()),
				0
			);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0,
			);
			assert_eq!(DexModule::provisioning_pool(SETUSD_DNAR_PAIR, ALICE), (0, 0));
			assert_eq!(DexModule::provisioning_pool(SETUSD_DNAR_PAIR, BOB), (0, 0));
			assert_eq!(
				DexModule::trading_pair_statuses(SETUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (0, 0),
					not_before: 10,
				})
			);

			Event::dex(crate::Event::ProvisioningToEnabled(
				SETUSD_DNAR_PAIR,
				1_000_000_000_000_000u128,
				2_000_000_000_000_000u128,
				4_000_000_000_000_000u128,
			));
		});
}

#[test]
fn get_liquidity_work() {
	ExtBuilder::default().build().execute_with(|| {
		LiquidityPool::<Runtime>::insert(SETUSD_DNAR_PAIR, (1000, 20));
		assert_eq!(DexModule::liquidity_pool(SETUSD_DNAR_PAIR), (1000, 20));
		assert_eq!(DexModule::get_liquidity(SETUSD, DNAR), (0, 0));
		assert_eq!(DexModule::get_liquidity(DNAR, SETUSD), (0, 0));
	});
}

#[test]
fn get_target_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(DexModule::get_target_amount(10000, 0, 1000), 0);
		assert_eq!(DexModule::get_target_amount(0, 20000, 1000), 0);
		assert_eq!(DexModule::get_target_amount(10000, 20000, 0), 0);
		assert_eq!(DexModule::get_target_amount(10000, 1, 1000000), 0);
		assert_eq!(DexModule::get_target_amount(10000, 20000, 10000), 9949);
		assert_eq!(DexModule::get_target_amount(10000, 20000, 1000), 1801);
	});
}

#[test]
fn get_supply_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(DexModule::get_supply_amount(10000, 0, 1000), 0);
		assert_eq!(DexModule::get_supply_amount(0, 20000, 1000), 0);
		assert_eq!(DexModule::get_supply_amount(10000, 20000, 0), 0);
		assert_eq!(DexModule::get_supply_amount(10000, 1, 1), 0);
		assert_eq!(DexModule::get_supply_amount(10000, 20000, 9949), 9999);
		assert_eq!(DexModule::get_target_amount(10000, 20000, 9999), 9949);
		assert_eq!(DexModule::get_supply_amount(10000, 20000, 1801), 1000);
		assert_eq!(DexModule::get_target_amount(10000, 20000, 1000), 1801);
	});
}

#[test]
fn get_target_amounts_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(SETUSD_DNAR_PAIR, (50000, 10000));
			LiquidityPool::<Runtime>::insert(SETUSD_SETHEUM_PAIR, (100000, 10));
			assert_noop!(
				DexModule::get_target_amounts(&vec![DNAR], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				DexModule::get_target_amounts(&vec![DNAR, SETUSD, SETHEUM, DNAR], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				DexModule::get_target_amounts(&vec![DNAR, SETUSD], 10000),
				Error::<Runtime>::InsufficientLiquidity,
			);
			assert_noop!(
				DexModule::get_target_amounts(&vec![DNAR, SETUSD, SETHEUM], 10000),
				Error::<Runtime>::InsufficientLiquidity,
			);
			assert_noop!(
				DexModule::get_target_amounts(&vec![DNAR, SETUSD, SETHEUM], 100),
				Error::<Runtime>::InsufficientLiquidity,
			);
			assert_noop!(
				DexModule::get_target_amounts(&vec![DNAR, SETHEUM], 100),
				Error::<Runtime>::InsufficientLiquidity,
			);
		});
}

#[test]
fn calculate_amount_for_big_number_work() {
	ExtBuilder::default().build().execute_with(|| {
		LiquidityPool::<Runtime>::insert(
			SETUSD_DNAR_PAIR,
			(171_000_000_000_000_000_000_000, 56_000_000_000_000_000_000_000),
		);
		assert_eq!(
			DexModule::get_supply_amount(
				171_000_000_000_000_000_000_000,
				56_000_000_000_000_000_000_000,
				1_000_000_000_000_000_000_000
			),
			3_140_495_867_768_595_041_323
		);
		assert_eq!(
			DexModule::get_target_amount(
				171_000_000_000_000_000_000_000,
				56_000_000_000_000_000_000_000,
				3_140_495_867_768_595_041_323
			),
			1_000_000_000_000_000_000_000
		);
	});
}

#[test]
fn get_supply_amounts_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(SETUSD_DNAR_PAIR, (50000, 10000));
			LiquidityPool::<Runtime>::insert(SETUSD_SETHEUM_PAIR, (100000, 10));
			assert_noop!(
				DexModule::get_supply_amounts(&vec![DNAR], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				DexModule::get_supply_amounts(&vec![DNAR, SETUSD, SETHEUM, DNAR], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				DexModule::get_supply_amounts(&vec![DNAR, SETUSD], 25000),
				Error::<Runtime>::InsufficientLiquidity,
			);
			assert_noop!(
				DexModule::get_supply_amounts(&vec![DNAR, SETUSD, SETHEUM], 10000),
				Error::<Runtime>::InsufficientLiquidity,
			);
			assert_noop!(
				DexModule::get_supply_amounts(&vec![DNAR, SETHEUM], 10000),
				Error::<Runtime>::InsufficientLiquidity,
			);
		});
}

#[test]
fn _swap_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(SETUSD_DNAR_PAIR, (50000, 10000));

			assert_eq!(DexModule::get_liquidity(SETUSD, DNAR), (0, 0));
			assert_ok!(DexModule::_swap(SETUSD, DNAR, 1000, 1000));
			assert_eq!(DexModule::get_liquidity(SETUSD, DNAR), (1000, 0));
			assert_ok!(DexModule::_swap(DNAR, SETUSD, 100, 800));
			assert_eq!(DexModule::get_liquidity(SETUSD, DNAR), (200, 100));
		});
}

#[test]
fn _swap_by_path_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(SETUSD_DNAR_PAIR, (50000, 10000));
			LiquidityPool::<Runtime>::insert(SETUSD_SETHEUM_PAIR, (100000, 10));

			assert_eq!(DexModule::get_liquidity(SETUSD, DNAR), (0, 0));
			assert_eq!(DexModule::get_liquidity(SETUSD, SETHEUM), (0, 0));
			assert_ok!(DexModule::_swap_by_path(&vec![DNAR, SETUSD], &vec![10000, 25000]));
			assert_eq!(DexModule::get_liquidity(SETUSD, DNAR), (0, 10000));
			assert_ok!(DexModule::_swap_by_path(&vec![DNAR, SETUSD, SETHEUM], &vec![4000, 10000, 2]));
			assert_eq!(DexModule::get_liquidity(SETUSD, DNAR), (0, 14000));
			assert_eq!(DexModule::get_liquidity(SETUSD, SETHEUM), (10000, 0));
		});
}

#[test]
fn add_liquidity_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_noop!(
				DexModule::add_liquidity(Origin::signed(ALICE), SETUSD, DNAR, 0, 100_000_000, 0),
				Error::<Runtime>::InvalidLiquidityIncrement
			);

			assert_eq!(DexModule::get_liquidity(SETUSD, DNAR), (0, 0));
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 0);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000);

			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				DNAR,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
			));
			Event::dex(crate::Event::AddLiquidity(
				ALICE,
				SETUSD,
				5_000_000_000_000,
				DNAR,
				1_000_000_000_000,
				10_000_000_000_000,
			));
			assert_eq!(
				DexModule::get_liquidity(SETUSD, DNAR),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_999_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000);

			assert_eq!(
				DexModule::get_liquidity(SETUSD, DNAR),
				(5000000000000, 1000000000000)
			);
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 5000000000000);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1000000000000000000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1000000000000000000);
		});
}

#[test]
fn remove_liquidity_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				DNAR,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
			));
			assert_noop!(
				DexModule::remove_liquidity(
					Origin::signed(ALICE),
					SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(),
					DNAR,
					100_000_000,
					0,
					0,
				),
				Error::<Runtime>::InvalidCurrencyId
			);

			assert_eq!(
				DexModule::get_liquidity(SETUSD, DNAR),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_999_000_000_000_000);

			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_999_000_000_000_000);

			assert_ok!(DexModule::remove_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				DNAR,
				2_000_000_000_000,
				0,
				0,
			));
			Event::dex(crate::Event::RemoveLiquidity(
				ALICE,
				SETUSD,
				1_000_000_000_000,
				DNAR,
				200_000_000_000,
				2_000_000_000_000,
			));

			assert_eq!(DexModule::get_liquidity(SETUSD, DNAR), (4000000000000, 800000000000));
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 4000000000000);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 800000000000);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 999996000000000000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999999200000000000);

			assert_ok!(DexModule::add_liquidity(
				Origin::signed(BOB),
				SETUSD,
				DNAR,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
			));
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_ok!(DexModule::remove_liquidity(
				Origin::signed(BOB),
				SETUSD,
				DNAR,
				2_000_000_000_000,
				0,
				0,
			));
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
		});
}

#[test]
fn do_swap_with_exact_supply_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				DNAR,
				500_000_000_000_000,
				100_000_000_000_000,
				0,
			));
			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				SETHEUM,
				100_000_000_000_000,
				10_000_000_000,
				0,
			));

			assert_eq!(
				DexModule::get_liquidity(SETUSD, DNAR),
				(500_000_000_000_000, 100_000_000_000_000)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, SETHEUM),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				600_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 100_000_000_000_000);
			assert_eq!(Tokens::free_balance(SETHEUM, &DexModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(SETHEUM, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				DexModule::do_swap_with_exact_supply(
					&BOB,
					&[DNAR, SETUSD],
					100_000_000_000_000,
					250_000_000_000_000,
				),
				Error::<Runtime>::InsufficientTargetAmount
			);
			assert_noop!(
				DexModule::do_swap_with_exact_supply(&BOB, &[DNAR, SETUSD, SETHEUM, DNAR], 100_000_000_000_000, 0),
				Error::<Runtime>::InvalidTradingPathLength,
			);

			assert_ok!(DexModule::do_swap_with_exact_supply(
				&BOB,
				&[DNAR, SETUSD],
				100_000_000_000_000,
				200_000_000_000_000,
			));
			Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, SETUSD],
				100_000_000_000_000,
				248_743_718_592_964,
			));
			assert_eq!(
				DexModule::get_liquidity(SETUSD, DNAR),
				(251_256_281_407_036, 200_000_000_000_000)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, SETHEUM),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				351_256_281_407_036
			);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 200_000_000_000_000);
			assert_eq!(Tokens::free_balance(SETHEUM, &DexModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1000249373433583959);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_900_000_000_000_000);
			assert_eq!(Tokens::free_balance(SETHEUM, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(DexModule::do_swap_with_exact_supply(
				&BOB,
				&[DNAR, SETUSD, SETHEUM],
				200_000_000_000_000,
				1,
			));
			let swap_event_2 = Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, SETUSD, SETHEUM],
				200_000_000_000_000,
				5_530_663_837,
			));
			assert!(System::events().iter().any(|record| record.event == swap_event_2));

			assert_eq!(
				DexModule::get_liquidity(SETUSD, DNAR),
				(126_259_437_892_983, 400_000_000_000_000)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, SETHEUM),
				(224_996_843_514_053, 4_469_336_163)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				351_256_281_407_036
			);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 400_000_000_000_000);
			assert_eq!(Tokens::free_balance(SETHEUM, &DexModule::account_id()), 4_469_336_163);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1000249373433583959);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_700_000_000_000_000);
			assert_eq!(Tokens::free_balance(SETHEUM, &BOB), 1_000_000_005_530_663_837);
		});
}

#[test]
fn do_swap_with_exact_target_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				DNAR,
				500_000_000_000_000,
				100_000_000_000_000,
				0,
			));
			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				SETHEUM,
				100_000_000_000_000,
				10_000_000_000,
				0,
			));

			assert_eq!(
				DexModule::get_liquidity(SETUSD, DNAR),
				(500_000_000_000_000, 100_000_000_000_000)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, SETHEUM),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				600_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 100_000_000_000_000);
			assert_eq!(Tokens::free_balance(SETHEUM, &DexModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(SETHEUM, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				DexModule::do_swap_with_exact_target(
					&BOB,
					&[DNAR, SETUSD],
					250_000_000_000_000,
					100_000_000_000_000,
				),
				Error::<Runtime>::ExcessiveSupplyAmount
			);
			assert_ok!(DexModule::do_swap_with_exact_target(
				&BOB,
				&[DNAR, SETUSD],
				250_000_000_000_000,
				200_000_000_000_000,
			));
			Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, SETUSD],
				101_010_101_010_102,
				250_000_000_000_000,
			));
			assert_noop!(
				DexModule::do_swap_with_exact_target(
					&BOB,
					&[DNAR, SETUSD, SETHEUM, DNAR],
					250_000_000_000_000,
					200_000_000_000_000,
				),
				Error::<Runtime>::InvalidTradingPathLength,
			);

			assert_eq!(
				DexModule::get_liquidity(SETUSD, DNAR),
				(250000000000001, 200502512562815)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, SETHEUM),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				350_000_000_000_001
			);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 200502512562815);
			assert_eq!(Tokens::free_balance(SETHEUM, &DexModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_250_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999899497487437185);
			assert_eq!(Tokens::free_balance(SETHEUM, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(DexModule::do_swap_with_exact_target(
				&BOB,
				&[DNAR, SETUSD, SETHEUM],
				5_000_000_000,
				2_000_000_000_000_000,
			));
			Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, SETUSD, SETHEUM],
				137_654_580_386_993,
				5_000_000_000,
			));

			assert_eq!(
				DexModule::get_liquidity(SETUSD, DNAR),
				(148_989_898_989_899, 337809489150946)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, SETHEUM),
				(201_010_101_010_102, 5_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				350_000_000_000_001
			);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 337809489150946);
			assert_eq!(Tokens::free_balance(SETHEUM, &DexModule::account_id()), 5_000_000_000);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_250_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999762190510849054);
			assert_eq!(Tokens::free_balance(SETHEUM, &BOB), 1_000_000_005_000_000_000);
		});
}

#[test]
fn initialize_added_liquidity_pools_genesis_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.initialize_added_liquidity_pools(ALICE)
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_eq!(DexModule::get_liquidity(SETUSD, DNAR), (1500000, 3000000));
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 3000000);
			assert_eq!(Tokens::free_balance(DNAR, &DexModule::account_id()), 4500000);
			assert_eq!(
				Tokens::free_balance(SETUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
		});
}
