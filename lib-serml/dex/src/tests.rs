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

//! Unit tests for the dex module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{
	Dex, Event, ExtBuilder, ListingOrigin, Origin, Runtime, System, Tokens, DNAR, ALICE, USDJ, USDJ_DNAR_PAIR,
	USDJ_JCHF_PAIR, BOB, JCHF,
};
use orml_traits::MultiReservableCurrency;
use sp_runtime::traits::BadOrigin;

#[test]
fn enable_new_trading_pair_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			Dex::enable_trading_pair(Origin::signed(ALICE), USDJ, DNAR),
			BadOrigin
		);

		assert_eq!(
			Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);
		assert_ok!(Dex::enable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			USDJ,
			DNAR
		));
		assert_eq!(
			Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
			TradingPairStatus::<_, _>::Enabled
		);

		System::assert_last_event(Event::dex(crate::Event::EnableTradingPair(USDJ_DNAR_PAIR)));

		assert_noop!(
			Dex::enable_trading_pair(Origin::signed(ListingOrigin::get()), DNAR, USDJ),
			Error::<Runtime>::MustBeNotEnabled
		);
	});
}

#[test]
fn list_new_trading_pair_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			Dex::list_trading_pair(
				Origin::signed(ALICE),
				USDJ,
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
			Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);
		assert_ok!(Dex::list_trading_pair(
			Origin::signed(ListingOrigin::get()),
			USDJ,
			DNAR,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_eq!(
			Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
			TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);

		System::assert_last_event(Event::dex(crate::Event::ListTradingPair(USDJ_DNAR_PAIR)));
		assert_noop!(
			Dex::list_trading_pair(
				Origin::signed(ListingOrigin::get()),
				DNAR,
				DNAR,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			Error::<Runtime>::NotAllowedList
		);

		assert_noop!(
			Dex::list_trading_pair(
				Origin::signed(ListingOrigin::get()),
				USDJ,
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

		assert_ok!(Dex::enable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			USDJ,
			DNAR
		));
		assert_eq!(
			Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
			TradingPairStatus::<_, _>::Enabled
		);

		assert_noop!(
			Dex::disable_trading_pair(Origin::signed(ALICE), USDJ, DNAR),
			BadOrigin
		);

		assert_ok!(Dex::disable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			USDJ,
			DNAR
		));
		assert_eq!(
			Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);

		System::assert_last_event(Event::dex(crate::Event::DisableTradingPair(USDJ_DNAR_PAIR)));
		assert_noop!(
			Dex::disable_trading_pair(Origin::signed(ListingOrigin::get()), USDJ, DNAR),
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

			assert_ok!(Dex::add_liquidity(
				Origin::signed(ALICE),
				USDJ,
				DNAR,
				5_000_000_000_000u128,
				0,
				false
			));
			assert_ok!(Dex::add_liquidity(
				Origin::signed(BOB),
				USDJ,
				DNAR,
				5_000_000_000_000u128,
				1_000_000_000_000u128,
				false
			));

			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 999_995_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 999_995_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_999_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(USDJ, &Dex::account_id()),
				10_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(DNAR, &Dex::account_id()),
				1_000_000_000_000u128
			);
			assert_eq!(
				Dex::provisioning_pool(USDJ_DNAR_PAIR, ALICE),
				(5_000_000_000_000u128, 0)
			);
			assert_eq!(
				Dex::provisioning_pool(USDJ_DNAR_PAIR, BOB),
				(5_000_000_000_000u128, 1_000_000_000_000u128)
			);
			assert_eq!(
				Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (10_000_000_000_000u128, 1_000_000_000_000u128),
					not_before: 10,
				})
			);
			let alice_ref_count_0 = System::consumers(&ALICE);
			let bob_ref_count_0 = System::consumers(&BOB);

			assert_ok!(Dex::disable_trading_pair(
				Origin::signed(ListingOrigin::get()),
				USDJ,
				DNAR
			));
			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(USDJ, &Dex::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 0);
			assert_eq!(Dex::provisioning_pool(USDJ_DNAR_PAIR, ALICE), (0, 0));
			assert_eq!(Dex::provisioning_pool(USDJ_DNAR_PAIR, BOB), (0, 0));
			assert_eq!(
				Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
				TradingPairStatus::<_, _>::NotEnabled
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
				Dex::add_liquidity(
					Origin::signed(ALICE),
					USDJ,
					DNAR,
					4_999_999_999_999u128,
					999_999_999_999u128,
					false
				),
				Error::<Runtime>::InvalidContributionIncrement
			);

			// alice add provision
			assert_eq!(
				Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (0, 0),
					not_before: 10,
				})
			);
			assert_eq!(Dex::provisioning_pool(USDJ_DNAR_PAIR, ALICE), (0, 0));
			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(USDJ, &Dex::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 0);
			let alice_ref_count_0 = System::consumers(&ALICE);

			assert_ok!(Dex::add_liquidity(
				Origin::signed(ALICE),
				USDJ,
				DNAR,
				5_000_000_000_000u128,
				0,
				false
			));
			assert_eq!(
				Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (5_000_000_000_000u128, 0),
					not_before: 10,
				})
			);
			assert_eq!(
				Dex::provisioning_pool(USDJ_DNAR_PAIR, ALICE),
				(5_000_000_000_000u128, 0)
			);
			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 999_995_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(USDJ, &Dex::account_id()),
				5_000_000_000_000u128
			);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 0);
			let alice_ref_count_1 = System::consumers(&ALICE);
			assert_eq!(alice_ref_count_1, alice_ref_count_0 + 1);
			System::assert_last_event(Event::dex(crate::Event::AddProvision(
				ALICE,
				USDJ,
				5_000_000_000_000u128,
				DNAR,
				0,
			)));

			// bob add provision
			assert_eq!(Dex::provisioning_pool(USDJ_DNAR_PAIR, BOB), (0, 0));
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000u128);
			let bob_ref_count_0 = System::consumers(&BOB);

			assert_ok!(Dex::add_liquidity(
				Origin::signed(BOB),
				DNAR,
				USDJ,
				1_000_000_000_000_000u128,
				0,
				false
			));
			assert_eq!(
				Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (5_000_000_000_000u128, 1_000_000_000_000_000u128),
					not_before: 10,
				})
			);
			assert_eq!(
				Dex::provisioning_pool(USDJ_DNAR_PAIR, BOB),
				(0, 1_000_000_000_000_000u128)
			);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_000_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(USDJ, &Dex::account_id()),
				5_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(DNAR, &Dex::account_id()),
				1_000_000_000_000_000u128
			);
			let bob_ref_count_1 = System::consumers(&BOB);
			assert_eq!(bob_ref_count_1, bob_ref_count_0 + 1);
			System::assert_last_event(Event::dex(crate::Event::AddProvision(
				BOB,
				USDJ,
				0,
				DNAR,
				1_000_000_000_000_000u128,
			)));

			// alice add provision again and trigger trading pair convert to Enabled from
			// Provisioning
			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 999_995_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(
				Tokens::total_issuance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap()),
				0
			);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);

			System::set_block_number(10);
			assert_ok!(Dex::add_liquidity(
				Origin::signed(ALICE),
				USDJ,
				DNAR,
				995_000_000_000_000u128,
				1_000_000_000_000_000u128,
				false
			));
			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 999_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_000_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(USDJ, &Dex::account_id()),
				1_000_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(DNAR, &Dex::account_id()),
				2_000_000_000_000_000u128
			);
			assert_eq!(
				Tokens::total_issuance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap()),
				4_000_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				3_000_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				1_000_000_000_000_000,
			);
			assert_eq!(Dex::provisioning_pool(USDJ_DNAR_PAIR, ALICE), (0, 0));
			assert_eq!(Dex::provisioning_pool(USDJ_DNAR_PAIR, BOB), (0, 0));
			assert_eq!(
				Dex::trading_pair_statuses(USDJ_DNAR_PAIR),
				TradingPairStatus::<_, _>::Enabled
			);

			System::assert_last_event(Event::dex(crate::Event::ProvisioningToEnabled(
				USDJ_DNAR_PAIR,
				1_000_000_000_000_000u128,
				2_000_000_000_000_000u128,
				4_000_000_000_000_000u128,
			)));
		});
}

#[test]
fn get_liquidity_work() {
	ExtBuilder::default().build().execute_with(|| {
		LiquidityPool::<Runtime>::insert(USDJ_DNAR_PAIR, (1000, 20));
		assert_eq!(Dex::liquidity_pool(USDJ_DNAR_PAIR), (1000, 20));
		assert_eq!(Dex::get_liquidity(USDJ, DNAR), (1000, 20));
		assert_eq!(Dex::get_liquidity(DNAR, USDJ), (20, 1000));
	});
}

#[test]
fn get_target_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Dex::get_target_amount(10000, 0, 1000), 0);
		assert_eq!(Dex::get_target_amount(0, 20000, 1000), 0);
		assert_eq!(Dex::get_target_amount(10000, 20000, 0), 0);
		assert_eq!(Dex::get_target_amount(10000, 1, 1000000), 0);
		assert_eq!(Dex::get_target_amount(10000, 20000, 10000), 9949);
		assert_eq!(Dex::get_target_amount(10000, 20000, 1000), 1801);
	});
}

#[test]
fn get_supply_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Dex::get_supply_amount(10000, 0, 1000), 0);
		assert_eq!(Dex::get_supply_amount(0, 20000, 1000), 0);
		assert_eq!(Dex::get_supply_amount(10000, 20000, 0), 0);
		assert_eq!(Dex::get_supply_amount(10000, 1, 1), 0);
		assert_eq!(Dex::get_supply_amount(10000, 20000, 9949), 9999);
		assert_eq!(Dex::get_target_amount(10000, 20000, 9999), 9949);
		assert_eq!(Dex::get_supply_amount(10000, 20000, 1801), 1000);
		assert_eq!(Dex::get_target_amount(10000, 20000, 1000), 1801);
	});
}

#[test]
fn get_target_amounts_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(USDJ_DNAR_PAIR, (50000, 10000));
			LiquidityPool::<Runtime>::insert(USDJ_JCHF_PAIR, (100000, 10));
			assert_noop!(
				Dex::get_target_amounts(&vec![DNAR], 10000, None),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				Dex::get_target_amounts(&vec![DNAR, USDJ, JCHF], 10000, None),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				Dex::get_target_amounts(&vec![DNAR, USDJ], 10000, None),
				Error::<Runtime>::MustBeEnabled,
			);
			assert_eq!(
				Dex::get_target_amounts(&vec![DNAR, USDJ], 10000, None),
				Ok(vec![10000, 24874])
			);
			assert_eq!(
				Dex::get_target_amounts(&vec![DNAR, USDJ], 10000, Ratio::checked_from_rational(50, 100)),
				Ok(vec![10000, 24874])
			);
			assert_noop!(
				Dex::get_target_amounts(&vec![DNAR, USDJ], 10000, Ratio::checked_from_rational(49, 100)),
				Error::<Runtime>::ExceedPriceImpactLimit,
			);
			assert_eq!(
				Dex::get_target_amounts(&vec![DNAR, USDJ, JCHF], 10000, None),
				Ok(vec![10000, 24874, 1])
			);
			assert_noop!(
				Dex::get_target_amounts(&vec![DNAR, USDJ, JCHF], 100, None),
				Error::<Runtime>::ZeroTargetAmount,
			);
			assert_noop!(
				Dex::get_target_amounts(&vec![DNAR, JCHF], 100, None),
				Error::<Runtime>::InsufficientLiquidity,
			);
		});
}

#[test]
fn calculate_amount_for_big_number_work() {
	ExtBuilder::default().build().execute_with(|| {
		LiquidityPool::<Runtime>::insert(
			USDJ_DNAR_PAIR,
			(171_000_000_000_000_000_000_000, 56_000_000_000_000_000_000_000),
		);
		assert_eq!(
			Dex::get_supply_amount(
				171_000_000_000_000_000_000_000,
				56_000_000_000_000_000_000_000,
				1_000_000_000_000_000_000_000
			),
			3_140_495_867_768_595_041_323
		);
		assert_eq!(
			Dex::get_target_amount(
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
			LiquidityPool::<Runtime>::insert(USDJ_DNAR_PAIR, (50000, 10000));
			LiquidityPool::<Runtime>::insert(USDJ_JCHF_PAIR, (100000, 10));
			assert_noop!(
				Dex::get_supply_amounts(&vec![DNAR], 10000, None),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				Dex::get_supply_amounts(&vec![DNAR, USDJ, JCHF], 10000, None),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				Dex::get_supply_amounts(&vec![DNAR, USDJ], 10000, None),
				Error::<Runtime>::MustBeEnabled,
			);
			assert_eq!(
				Dex::get_supply_amounts(&vec![DNAR, USDJ], 24874, None),
				Ok(vec![10000, 24874])
			);
			assert_eq!(
				Dex::get_supply_amounts(&vec![DNAR, USDJ], 25000, Ratio::checked_from_rational(50, 100)),
				Ok(vec![10102, 25000])
			);
			assert_noop!(
				Dex::get_supply_amounts(&vec![DNAR, USDJ], 25000, Ratio::checked_from_rational(49, 100)),
				Error::<Runtime>::ExceedPriceImpactLimit,
			);
			assert_noop!(
				Dex::get_supply_amounts(&vec![DNAR, USDJ, JCHF], 10000, None),
				Error::<Runtime>::ZeroSupplyAmount,
			);
			assert_noop!(
				Dex::get_supply_amounts(&vec![DNAR, JCHF], 10000, None),
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
			LiquidityPool::<Runtime>::insert(USDJ_DNAR_PAIR, (50000, 10000));

			assert_eq!(Dex::get_liquidity(USDJ, DNAR), (50000, 10000));
			Dex::_swap(USDJ, DNAR, 1000, 1000);
			assert_eq!(Dex::get_liquidity(USDJ, DNAR), (51000, 9000));
			Dex::_swap(DNAR, USDJ, 100, 800);
			assert_eq!(Dex::get_liquidity(USDJ, DNAR), (50200, 9100));
		});
}

#[test]
fn _swap_by_path_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(USDJ_DNAR_PAIR, (50000, 10000));
			LiquidityPool::<Runtime>::insert(USDJ_JCHF_PAIR, (100000, 10));

			assert_eq!(Dex::get_liquidity(USDJ, DNAR), (50000, 10000));
			assert_eq!(Dex::get_liquidity(USDJ, JCHF), (100000, 10));
			Dex::_swap_by_path(&vec![DNAR, USDJ], &vec![10000, 25000]);
			assert_eq!(Dex::get_liquidity(USDJ, DNAR), (25000, 20000));
			Dex::_swap_by_path(&vec![DNAR, USDJ, JCHF], &vec![4000, 10000, 2]);
			assert_eq!(Dex::get_liquidity(USDJ, DNAR), (15000, 24000));
			assert_eq!(Dex::get_liquidity(USDJ, JCHF), (110000, 8));
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
				Dex::add_liquidity(Origin::signed(ALICE), DNAR, USDJ, 100_000_000, 100_000_000, false),
				Error::<Runtime>::NotEnabledTradingPair
			);
			assert_noop!(
				Dex::add_liquidity(Origin::signed(ALICE), USDJ, DNAR, 0, 100_000_000, false),
				Error::<Runtime>::InvalidLiquidityIncrement
			);

			assert_eq!(Dex::get_liquidity(USDJ, DNAR), (0, 0));
			assert_eq!(Tokens::free_balance(USDJ, &Dex::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 0);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000);

			assert_ok!(Dex::add_liquidity(
				Origin::signed(ALICE),
				USDJ,
				DNAR,
				5_000_000_000_000,
				1_000_000_000_000,
				false,
			));
			System::assert_last_event(Event::dex(crate::Event::AddLiquidity(
				ALICE,
				USDJ,
				5_000_000_000_000,
				DNAR,
				1_000_000_000_000,
				10_000_000_000_000,
			)));
			assert_eq!(
				Dex::get_liquidity(USDJ, DNAR),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(USDJ, &Dex::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				10_000_000_000_000
			);
			assert_eq!(
				Tokens::reserved_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_999_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				Dex::add_liquidity(
					Origin::signed(BOB),
					USDJ,
					DNAR,
					50_000_000_000_000,
					8_000_000_000_000,
					80_000_000_000_001,
					true,
				),
				Error::<Runtime>::UnacceptableShareIncrement
			);
			assert_ok!(Dex::add_liquidity(
					Origin::signed(BOB),
					USDJ,
					DNAR,
					50_000_000_000_000,
					8_000_000_000_000,
					80_000_000_000_001,
					true,
				));
			System::assert_last_event(Event::dex(crate::Event::AddLiquidity(
				BOB,
				USDJ,
				40_000_000_000_000,
				DNAR,
				8_000_000_000_000,
				80_000_000_000_000,
			)));
			assert_eq!(
				Dex::get_liquidity(USDJ, DNAR),
				(45_000_000_000_000, 9_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(USDJ, &Dex::account_id()), 45_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 9_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				80_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 999_960_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_992_000_000_000_000);
		});
}

#[test]
fn remove_liquidity_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(Dex::add_liquidity(
				Origin::signed(ALICE),
				USDJ,
				DNAR,
				5_000_000_000_000,
				1_000_000_000_000,
				false
			));
			assert_noop!(
				Dex::remove_liquidity(
					Origin::signed(ALICE),
					USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(),
					DNAR,
					100_000_000,
					0,
					0,
					false,
				),
				Error::<Runtime>::InvalidCurrencyId
			);

			assert_eq!(
				Dex::get_liquidity(USDJ, DNAR),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(USDJ, &Dex::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				10_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_999_000_000_000_000);

			assert_noop!(
				Dex::remove_liquidity(
					Origin::signed(ALICE),
					USDJ,
					DNAR,
					8_000_000_000_000,
					4_000_000_000_001,
					800_000_000_000,
					false,
				),
				Error::<Runtime>::UnacceptableLiquidityWithdrawn
			);
			assert_noop!(
				Dex::remove_liquidity(
					Origin::signed(ALICE),
					USDJ,
					DNAR,
					8_000_000_000_000,
					4_000_000_000_000,
					800_000_000_001,
					false,
				),
				Error::<Runtime>::UnacceptableLiquidityWithdrawn
			);
			assert_ok!(Dex::remove_liquidity(
				Origin::signed(ALICE),
				USDJ,
				DNAR,
				8_000_000_000_000,
				4_000_000_000_000,
				800_000_000_000,
				false,
			));
			System::assert_last_event(Event::dex(crate::Event::RemoveLiquidity(
				ALICE,
				USDJ,
				4_000_000_000_000,
				DNAR,
				800_000_000_000,
				8_000_000_000_000,
			)));
			assert_eq!(
				Dex::get_liquidity(USDJ, DNAR),
				(1_000_000_000_000, 200_000_000_000)
			);
			assert_eq!(Tokens::free_balance(USDJ, &Dex::account_id()), 1_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 200_000_000_000);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				2_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 999_999_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_999_800_000_000_000);

			assert_ok!(Dex::remove_liquidity(
				Origin::signed(ALICE),
				USDJ,
				DNAR,
				2_000_000_000_000,
				0,
				0,
				false,
			));
			System::assert_last_event(Event::dex(crate::Event::RemoveLiquidity(
				ALICE,
				USDJ,
				1_000_000_000_000,
				DNAR,
				200_000_000_000,
				2_000_000_000_000,
			)));
			assert_eq!(Dex::get_liquidity(USDJ, DNAR), (0, 0));
			assert_eq!(Tokens::free_balance(USDJ, &Dex::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 0);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(USDJ, &ALICE), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000);

			assert_ok!(Dex::add_liquidity(
				Origin::signed(BOB),
				USDJ,
				DNAR,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
				true
			));
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				10_000_000_000_000
			);
			assert_ok!(Dex::remove_liquidity(
				Origin::signed(BOB),
				USDJ,
				DNAR,
				2_000_000_000_000,
				0,
				0,
				true,
			));
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				8_000_000_000_000
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

			assert_ok!(Dex::add_liquidity(
				Origin::signed(ALICE),
				USDJ,
				DNAR,
				500_000_000_000_000,
				100_000_000_000_000,
				0,
				false,
			));
			assert_ok!(Dex::add_liquidity(
				Origin::signed(ALICE),
				USDJ,
				JCHF,
				100_000_000_000_000,
				10_000_000_000,
				0,
				false,
			));

			assert_eq!(
				Dex::get_liquidity(USDJ, DNAR),
				(500_000_000_000_000, 100_000_000_000_000)
			);
			assert_eq!(
				Dex::get_liquidity(USDJ, JCHF),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(USDJ, &Dex::account_id()),
				600_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 100_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &Dex::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				Dex::do_swap_with_exact_supply(
					&BOB,
					&[DNAR, USDJ],
					100_000_000_000_000,
					250_000_000_000_000,
					None
				),
				Error::<Runtime>::InsufficientTargetAmount
			);
			assert_noop!(
				Dex::do_swap_with_exact_supply(
					&BOB,
					&[DNAR, USDJ],
					100_000_000_000_000,
					0,
					Ratio::checked_from_rational(10, 100)
				),
				Error::<Runtime>::ExceedPriceImpactLimit,
			);
			assert_noop!(
				Dex::do_swap_with_exact_supply(&BOB, &[DNAR, USDJ, JCHF], 100_000_000_000_000, 0, None),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				Dex::do_swap_with_exact_supply(&BOB, &[JCHF, DNAR], 100_000_000_000_000, 0, None),
				Error::<Runtime>::MustBeEnabled,
			);

			assert_ok!(Dex::do_swap_with_exact_supply(
				&BOB,
				&[DNAR, USDJ],
				100_000_000_000_000,
				200_000_000_000_000,
				None
			));
			System::assert_last_event(Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, USDJ],
				100_000_000_000_000,
				248_743_718_592_964,
			)));
			assert_eq!(
				Dex::get_liquidity(USDJ, DNAR),
				(251_256_281_407_036, 200_000_000_000_000)
			);
			assert_eq!(
				Dex::get_liquidity(USDJ, JCHF),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(USDJ, &Dex::account_id()),
				351_256_281_407_036
			);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 200_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &Dex::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 1_000_248_743_718_592_964);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_900_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(Dex::do_swap_with_exact_supply(
				&BOB,
				&[DNAR, USDJ, JCHF],
				200_000_000_000_000,
				1,
				None
			));
			System::assert_last_event(Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, USDJ, JCHF],
				200_000_000_000_000,
				5_530_663_837,
			)));
			assert_eq!(
				Dex::get_liquidity(USDJ, DNAR),
				(126_259_437_892_983, 400_000_000_000_000)
			);
			assert_eq!(
				Dex::get_liquidity(USDJ, JCHF),
				(224_996_843_514_053, 4_469_336_163)
			);
			assert_eq!(
				Tokens::free_balance(USDJ, &Dex::account_id()),
				351_256_281_407_036
			);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 400_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &Dex::account_id()), 4_469_336_163);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 1_000_248_743_718_592_964);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_700_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &BOB), 1_000_000_005_530_663_837);
		});
}

#[test]
fn do_swap_with_exact_target_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(Dex::add_liquidity(
				Origin::signed(ALICE),
				USDJ,
				DNAR,
				500_000_000_000_000,
				100_000_000_000_000,
				0,
				false,
			));
			assert_ok!(Dex::add_liquidity(
				Origin::signed(ALICE),
				USDJ,
				JCHF,
				100_000_000_000_000,
				10_000_000_000,
				0,
				false,
			));

			assert_eq!(
				Dex::get_liquidity(USDJ, DNAR),
				(500_000_000_000_000, 100_000_000_000_000)
			);
			assert_eq!(
				Dex::get_liquidity(USDJ, JCHF),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(USDJ, &Dex::account_id()),
				600_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 100_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &Dex::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				Dex::do_swap_with_exact_target(
					&BOB,
					&[DNAR, USDJ],
					250_000_000_000_000,
					100_000_000_000_000,
					None
				),
				Error::<Runtime>::ExcessiveSupplyAmount
			);
			assert_noop!(
				Dex::do_swap_with_exact_target(
					&BOB,
					&[DNAR, USDJ],
					250_000_000_000_000,
					200_000_000_000_000,
					Ratio::checked_from_rational(10, 100)
				),
				Error::<Runtime>::ExceedPriceImpactLimit,
			);
			assert_noop!(
				Dex::do_swap_with_exact_target(
					&BOB,
					&[DNAR, USDJ, JCHF],
					250_000_000_000_000,
					200_000_000_000_000,
					None
				),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				Dex::do_swap_with_exact_target(&BOB, &[JCHF, DNAR], 250_000_000_000_000, 200_000_000_000_000, None),
				Error::<Runtime>::MustBeEnabled,
			);

			assert_ok!(Dex::do_swap_with_exact_target(
				&BOB,
				&[DNAR, USDJ],
				250_000_000_000_000,
				200_000_000_000_000,
				None
			));
			System::assert_last_event(Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, USDJ],
				101_010_101_010_102,
				250_000_000_000_000,
			)));

			assert_eq!(
				Dex::get_liquidity(USDJ, DNAR),
				(250_000_000_000_000, 201_010_101_010_102)
			);
			assert_eq!(
				Dex::get_liquidity(USDJ, JCHF),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(USDJ, &Dex::account_id()),
				350_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 201_010_101_010_102);
			assert_eq!(Tokens::free_balance(JCHF, &Dex::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 1_000_250_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_898_989_898_989_898);
			assert_eq!(Tokens::free_balance(JCHF, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(Dex::do_swap_with_exact_target(
				&BOB,
				&[DNAR, USDJ, JCHF],
				5_000_000_000,
				2_000_000_000_000_000,
				None
			));
			System::assert_last_event(Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, USDJ, JCHF],
				137_654_580_386_993,
				5_000_000_000,
			)));
			assert_eq!(
				Dex::get_liquidity(USDJ, DNAR),
				(148_989_898_989_898, 338_664_681_397_095)
			);
			assert_eq!(
				Dex::get_liquidity(USDJ, JCHF),
				(201_010_101_010_102, 5_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(USDJ, &Dex::account_id()),
				350_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 338_664_681_397_095);
			assert_eq!(Tokens::free_balance(JCHF, &Dex::account_id()), 5_000_000_000);
			assert_eq!(Tokens::free_balance(USDJ, &BOB), 1_000_250_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_761_335_318_602_905);
			assert_eq!(Tokens::free_balance(JCHF, &BOB), 1_000_000_005_000_000_000);
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

			assert_eq!(Dex::get_liquidity(USDJ, DNAR), (1000000, 2000000));
			assert_eq!(Tokens::free_balance(USDJ, &Dex::account_id()), 2000000);
			assert_eq!(Tokens::free_balance(DNAR, &Dex::account_id()), 3000000);
			assert_eq!(
				Tokens::free_balance(USDJ_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				4000000
			);
		});
}
