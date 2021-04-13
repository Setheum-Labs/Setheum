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
	SettinDex, Event, ExtBuilder, ListingOrigin, Origin, Runtime, System, Tokens, DNAR, ALICE, JUSD, JUSD_DNAR_PAIR,
	JUSD_JCHF_PAIR, BOB, DNAR, JCHF,
};
use orml_traits::MultiReservableCurrency;
use sp_runtime::traits::BadOrigin;

#[test]
fn enable_new_trading_pair_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			SettinDex::enable_trading_pair(Origin::signed(ALICE), JUSD, DNAR),
			BadOrigin
		);

		assert_eq!(
			SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);
		assert_ok!(SettinDex::enable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			JUSD,
			DNAR
		));
		assert_eq!(
			SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::Enabled
		);

		let enable_trading_pair_event = Event::dex(crate::Event::EnableTradingPair(JUSD_DNAR_PAIR));
		assert!(System::events()
			.iter()
			.any(|record| record.event == enable_trading_pair_event));

		assert_noop!(
			SettinDex::enable_trading_pair(Origin::signed(ListingOrigin::get()), DNAR, JUSD),
			Error::<Runtime>::MustBeNotEnabled
		);
	});
}

#[test]
fn list_new_trading_pair_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			SettinDex::list_trading_pair(
				Origin::signed(ALICE),
				JUSD,
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
			SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);
		assert_ok!(SettinDex::list_trading_pair(
			Origin::signed(ListingOrigin::get()),
			JUSD,
			DNAR,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_eq!(
			SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);

		let list_trading_pair_event = Event::dex(crate::Event::ListTradingPair(JUSD_DNAR_PAIR));
		assert!(System::events()
			.iter()
			.any(|record| record.event == list_trading_pair_event));

		assert_noop!(
			SettinDex::list_trading_pair(
				Origin::signed(ListingOrigin::get()),
				JUSD,
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

		assert_ok!(SettinDex::enable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			JUSD,
			DNAR
		));
		assert_eq!(
			SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::Enabled
		);

		assert_noop!(
			SettinDex::disable_trading_pair(Origin::signed(ALICE), JUSD, DNAR),
			BadOrigin
		);

		assert_ok!(SettinDex::disable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			JUSD,
			DNAR
		));
		assert_eq!(
			SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
			TradingPairStatus::<_, _>::NotEnabled
		);

		let disable_trading_pair_event = Event::dex(crate::Event::DisableTradingPair(JUSD_DNAR_PAIR));
		assert!(System::events()
			.iter()
			.any(|record| record.event == disable_trading_pair_event));

		assert_noop!(
			SettinDex::disable_trading_pair(Origin::signed(ListingOrigin::get()), JUSD, DNAR),
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

			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(ALICE),
				JUSD,
				DNAR,
				5_000_000_000_000u128,
				0,
				false
			));
			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(BOB),
				JUSD,
				DNAR,
				5_000_000_000_000u128,
				1_000_000_000_000u128,
				false
			));

			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 999_995_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 999_995_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_999_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(JUSD, &SettinDex::account_id()),
				10_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(DNAR, &SettinDex::account_id()),
				1_000_000_000_000u128
			);
			assert_eq!(
				SettinDex::provisioning_pool(JUSD_DNAR_PAIR, ALICE),
				(5_000_000_000_000u128, 0)
			);
			assert_eq!(
				SettinDex::provisioning_pool(JUSD_DNAR_PAIR, BOB),
				(5_000_000_000_000u128, 1_000_000_000_000u128)
			);
			assert_eq!(
				SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (10_000_000_000_000u128, 1_000_000_000_000u128),
					not_before: 10,
				})
			);
			let alice_ref_count_0 = System::consumers(&ALICE);
			let bob_ref_count_0 = System::consumers(&BOB);

			assert_ok!(SettinDex::disable_trading_pair(
				Origin::signed(ListingOrigin::get()),
				JUSD,
				DNAR
			));
			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(JUSD, &SettinDex::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 0);
			assert_eq!(SettinDex::provisioning_pool(JUSD_DNAR_PAIR, ALICE), (0, 0));
			assert_eq!(SettinDex::provisioning_pool(JUSD_DNAR_PAIR, BOB), (0, 0));
			assert_eq!(
				SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
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
				SettinDex::add_liquidity(
					Origin::signed(ALICE),
					JUSD,
					DNAR,
					4_999_999_999_999u128,
					999_999_999_999u128,
					false
				),
				Error::<Runtime>::InvalidContributionIncrement
			);

			// alice add provision
			assert_eq!(
				SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (0, 0),
					not_before: 10,
				})
			);
			assert_eq!(SettinDex::provisioning_pool(JUSD_DNAR_PAIR, ALICE), (0, 0));
			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(JUSD, &SettinDex::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 0);
			let alice_ref_count_0 = System::consumers(&ALICE);

			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(ALICE),
				JUSD,
				DNAR,
				5_000_000_000_000u128,
				0,
				false
			));
			assert_eq!(
				SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (5_000_000_000_000u128, 0),
					not_before: 10,
				})
			);
			assert_eq!(
				SettinDex::provisioning_pool(JUSD_DNAR_PAIR, ALICE),
				(5_000_000_000_000u128, 0)
			);
			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 999_995_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(JUSD, &SettinDex::account_id()),
				5_000_000_000_000u128
			);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 0);
			let alice_ref_count_1 = System::consumers(&ALICE);
			assert_eq!(alice_ref_count_1, alice_ref_count_0 + 1);

			let add_provision_event_0 =
				Event::dex(crate::Event::AddProvision(ALICE, JUSD, 5_000_000_000_000u128, DNAR, 0));
			assert!(System::events()
				.iter()
				.any(|record| record.event == add_provision_event_0));

			// bob add provision
			assert_eq!(SettinDex::provisioning_pool(JUSD_DNAR_PAIR, BOB), (0, 0));
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000u128);
			let bob_ref_count_0 = System::consumers(&BOB);

			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(BOB),
				DNAR,
				JUSD,
				1_000_000_000_000_000u128,
				0,
				false
			));
			assert_eq!(
				SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Provisioning(TradingPairProvisionParameters {
					min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
					target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
					accumulated_provision: (5_000_000_000_000u128, 1_000_000_000_000_000u128),
					not_before: 10,
				})
			);
			assert_eq!(
				SettinDex::provisioning_pool(JUSD_DNAR_PAIR, BOB),
				(0, 1_000_000_000_000_000u128)
			);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 1_000_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_000_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(JUSD, &SettinDex::account_id()),
				5_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(DNAR, &SettinDex::account_id()),
				1_000_000_000_000_000u128
			);
			let bob_ref_count_1 = System::consumers(&BOB);
			assert_eq!(bob_ref_count_1, bob_ref_count_0 + 1);

			let add_provision_event_1 =
				Event::dex(crate::Event::AddProvision(BOB, JUSD, 0, DNAR, 1_000_000_000_000_000u128));
			assert!(System::events()
				.iter()
				.any(|record| record.event == add_provision_event_1));

			// alice add provision again and trigger trading pair convert to Enabled from
			// Provisioning
			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 999_995_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000u128);
			assert_eq!(
				Tokens::total_issuance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap()),
				0
			);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);

			System::set_block_number(10);
			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(ALICE),
				JUSD,
				DNAR,
				995_000_000_000_000u128,
				1_000_000_000_000_000u128,
				false
			));
			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 999_000_000_000_000_000u128);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_000_000_000_000_000u128);
			assert_eq!(
				Tokens::free_balance(JUSD, &SettinDex::account_id()),
				1_000_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(DNAR, &SettinDex::account_id()),
				2_000_000_000_000_000u128
			);
			assert_eq!(
				Tokens::total_issuance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap()),
				4_000_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				3_000_000_000_000_000u128
			);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				1_000_000_000_000_000,
			);
			assert_eq!(SettinDex::provisioning_pool(JUSD_DNAR_PAIR, ALICE), (0, 0));
			assert_eq!(SettinDex::provisioning_pool(JUSD_DNAR_PAIR, BOB), (0, 0));
			assert_eq!(
				SettinDex::trading_pair_statuses(JUSD_DNAR_PAIR),
				TradingPairStatus::<_, _>::Enabled
			);

			let provisioning_to_enabled_event = Event::dex(crate::Event::ProvisioningToEnabled(
				JUSD_DNAR_PAIR,
				1_000_000_000_000_000u128,
				2_000_000_000_000_000u128,
				4_000_000_000_000_000u128,
			));
			assert!(System::events()
				.iter()
				.any(|record| record.event == provisioning_to_enabled_event));
		});
}

#[test]
fn get_liquidity_work() {
	ExtBuilder::default().build().execute_with(|| {
		LiquidityPool::<Runtime>::insert(JUSD_DNAR_PAIR, (1000, 20));
		assert_eq!(SettinDex::liquidity_pool(JUSD_DNAR_PAIR), (1000, 20));
		assert_eq!(SettinDex::get_liquidity(JUSD, DNAR), (1000, 20));
		assert_eq!(SettinDex::get_liquidity(DNAR, JUSD), (20, 1000));
	});
}

#[test]
fn get_target_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SettinDex::get_target_amount(10000, 0, 1000), 0);
		assert_eq!(SettinDex::get_target_amount(0, 20000, 1000), 0);
		assert_eq!(SettinDex::get_target_amount(10000, 20000, 0), 0);
		assert_eq!(SettinDex::get_target_amount(10000, 1, 1000000), 0);
		assert_eq!(SettinDex::get_target_amount(10000, 20000, 10000), 9949);
		assert_eq!(SettinDex::get_target_amount(10000, 20000, 1000), 1801);
	});
}

#[test]
fn get_supply_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SettinDex::get_supply_amount(10000, 0, 1000), 0);
		assert_eq!(SettinDex::get_supply_amount(0, 20000, 1000), 0);
		assert_eq!(SettinDex::get_supply_amount(10000, 20000, 0), 0);
		assert_eq!(SettinDex::get_supply_amount(10000, 1, 1), 0);
		assert_eq!(SettinDex::get_supply_amount(10000, 20000, 9949), 9999);
		assert_eq!(SettinDex::get_target_amount(10000, 20000, 9999), 9949);
		assert_eq!(SettinDex::get_supply_amount(10000, 20000, 1801), 1000);
		assert_eq!(SettinDex::get_target_amount(10000, 20000, 1000), 1801);
	});
}

#[test]
fn get_target_amounts_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(JUSD_DNAR_PAIR, (50000, 10000));
			LiquidityPool::<Runtime>::insert(JUSD_JCHF_PAIR, (100000, 10));
			assert_noop!(
				SettinDex::get_target_amounts(&vec![DNAR], 10000, None),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				SettinDex::get_target_amounts(&vec![DNAR, JUSD, JCHF, DNAR], 10000, None),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				SettinDex::get_target_amounts(&vec![DNAR, JUSD, DNAR], 10000, None),
				Error::<Runtime>::MustBeEnabled,
			);
			assert_eq!(
				SettinDex::get_target_amounts(&vec![DNAR, JUSD], 10000, None),
				Ok(vec![10000, 24874])
			);
			assert_eq!(
				SettinDex::get_target_amounts(&vec![DNAR, JUSD], 10000, Ratio::checked_from_rational(50, 100)),
				Ok(vec![10000, 24874])
			);
			assert_noop!(
				SettinDex::get_target_amounts(&vec![DNAR, JUSD], 10000, Ratio::checked_from_rational(49, 100)),
				Error::<Runtime>::ExceedPriceImpactLimit,
			);
			assert_eq!(
				SettinDex::get_target_amounts(&vec![DNAR, JUSD, JCHF], 10000, None),
				Ok(vec![10000, 24874, 1])
			);
			assert_noop!(
				SettinDex::get_target_amounts(&vec![DNAR, JUSD, JCHF], 100, None),
				Error::<Runtime>::ZeroTargetAmount,
			);
			assert_noop!(
				SettinDex::get_target_amounts(&vec![DNAR, JCHF], 100, None),
				Error::<Runtime>::InsufficientLiquidity,
			);
		});
}

#[test]
fn calculate_amount_for_big_number_work() {
	ExtBuilder::default().build().execute_with(|| {
		LiquidityPool::<Runtime>::insert(
			JUSD_DNAR_PAIR,
			(171_000_000_000_000_000_000_000, 56_000_000_000_000_000_000_000),
		);
		assert_eq!(
			SettinDex::get_supply_amount(
				171_000_000_000_000_000_000_000,
				56_000_000_000_000_000_000_000,
				1_000_000_000_000_000_000_000
			),
			3_140_495_867_768_595_041_323
		);
		assert_eq!(
			SettinDex::get_target_amount(
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
			LiquidityPool::<Runtime>::insert(JUSD_DNAR_PAIR, (50000, 10000));
			LiquidityPool::<Runtime>::insert(JUSD_JCHF_PAIR, (100000, 10));
			assert_noop!(
				SettinDex::get_supply_amounts(&vec![DNAR], 10000, None),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				SettinDex::get_supply_amounts(&vec![DNAR, JUSD, JCHF, DNAR], 10000, None),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				SettinDex::get_supply_amounts(&vec![DNAR, JUSD, DNAR], 10000, None),
				Error::<Runtime>::MustBeEnabled,
			);
			assert_eq!(
				SettinDex::get_supply_amounts(&vec![DNAR, JUSD], 24874, None),
				Ok(vec![10000, 24874])
			);
			assert_eq!(
				SettinDex::get_supply_amounts(&vec![DNAR, JUSD], 25000, Ratio::checked_from_rational(50, 100)),
				Ok(vec![10102, 25000])
			);
			assert_noop!(
				SettinDex::get_supply_amounts(&vec![DNAR, JUSD], 25000, Ratio::checked_from_rational(49, 100)),
				Error::<Runtime>::ExceedPriceImpactLimit,
			);
			assert_noop!(
				SettinDex::get_supply_amounts(&vec![DNAR, JUSD, JCHF], 10000, None),
				Error::<Runtime>::ZeroSupplyAmount,
			);
			assert_noop!(
				SettinDex::get_supply_amounts(&vec![DNAR, JCHF], 10000, None),
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
			LiquidityPool::<Runtime>::insert(JUSD_DNAR_PAIR, (50000, 10000));

			assert_eq!(SettinDex::get_liquidity(JUSD, DNAR), (50000, 10000));
			SettinDex::_swap(JUSD, DNAR, 1000, 1000);
			assert_eq!(SettinDex::get_liquidity(JUSD, DNAR), (51000, 9000));
			SettinDex::_swap(DNAR, JUSD, 100, 800);
			assert_eq!(SettinDex::get_liquidity(JUSD, DNAR), (50200, 9100));
		});
}

#[test]
fn _swap_by_path_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(JUSD_DNAR_PAIR, (50000, 10000));
			LiquidityPool::<Runtime>::insert(JUSD_JCHF_PAIR, (100000, 10));

			assert_eq!(SettinDex::get_liquidity(JUSD, DNAR), (50000, 10000));
			assert_eq!(SettinDex::get_liquidity(JUSD, JCHF), (100000, 10));
			SettinDex::_swap_by_path(&vec![DNAR, JUSD], &vec![10000, 25000]);
			assert_eq!(SettinDex::get_liquidity(JUSD, DNAR), (25000, 20000));
			SettinDex::_swap_by_path(&vec![DNAR, JUSD, JCHF], &vec![4000, 10000, 2]);
			assert_eq!(SettinDex::get_liquidity(JUSD, DNAR), (15000, 24000));
			assert_eq!(SettinDex::get_liquidity(JUSD, JCHF), (110000, 8));
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
				SettinDex::add_liquidity(Origin::signed(ALICE), DNAR, JUSD, 100_000_000, 100_000_000, false),
				Error::<Runtime>::NotEnabledTradingPair
			);
			assert_noop!(
				SettinDex::add_liquidity(Origin::signed(ALICE), JUSD, DNAR, 0, 100_000_000, false),
				Error::<Runtime>::InvalidLiquidityIncrement
			);

			assert_eq!(SettinDex::get_liquidity(JUSD, DNAR), (0, 0));
			assert_eq!(Tokens::free_balance(JUSD, &SettinDex::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 0);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000);

			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(ALICE),
				JUSD,
				DNAR,
				5_000_000_000_000,
				1_000_000_000_000,
				false,
			));
			let add_liquidity_event_1 = Event::dex(crate::Event::AddLiquidity(
				ALICE,
				JUSD,
				5_000_000_000_000,
				DNAR,
				1_000_000_000_000,
				10_000_000_000_000,
			));
			assert!(System::events()
				.iter()
				.any(|record| record.event == add_liquidity_event_1));

			assert_eq!(
				SettinDex::get_liquidity(JUSD, DNAR),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(JUSD, &SettinDex::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				10_000_000_000_000
			);
			assert_eq!(
				Tokens::reserved_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_999_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(BOB),
				JUSD,
				DNAR,
				50_000_000_000_000,
				8_000_000_000_000,
				true,
			));
			let add_liquidity_event_2 = Event::dex(crate::Event::AddLiquidity(
				BOB,
				JUSD,
				40_000_000_000_000,
				DNAR,
				8_000_000_000_000,
				80_000_000_000_000,
			));
			assert!(System::events()
				.iter()
				.any(|record| record.event == add_liquidity_event_2));

			assert_eq!(
				SettinDex::get_liquidity(JUSD, DNAR),
				(45_000_000_000_000, 9_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(JUSD, &SettinDex::account_id()), 45_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 9_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				80_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 999_960_000_000_000_000);
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

			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(ALICE),
				JUSD,
				DNAR,
				5_000_000_000_000,
				1_000_000_000_000,
				false
			));
			assert_noop!(
				SettinDex::remove_liquidity(
					Origin::signed(ALICE),
					JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(),
					DNAR,
					100_000_000,
					false,
				),
				Error::<Runtime>::InvalidCurrencyId
			);

			assert_eq!(
				SettinDex::get_liquidity(JUSD, DNAR),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(JUSD, &SettinDex::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				10_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_999_000_000_000_000);

			assert_ok!(SettinDex::remove_liquidity(
				Origin::signed(ALICE),
				JUSD,
				DNAR,
				8_000_000_000_000,
				false,
			));
			let remove_liquidity_event_1 = Event::dex(crate::Event::RemoveLiquidity(
				ALICE,
				JUSD,
				4_000_000_000_000,
				DNAR,
				800_000_000_000,
				8_000_000_000_000,
			));
			assert!(System::events()
				.iter()
				.any(|record| record.event == remove_liquidity_event_1));

			assert_eq!(
				SettinDex::get_liquidity(JUSD, DNAR),
				(1_000_000_000_000, 200_000_000_000)
			);
			assert_eq!(Tokens::free_balance(JUSD, &SettinDex::account_id()), 1_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 200_000_000_000);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				2_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 999_999_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 999_999_800_000_000_000);

			assert_ok!(SettinDex::remove_liquidity(
				Origin::signed(ALICE),
				JUSD,
				DNAR,
				2_000_000_000_000,
				false,
			));
			let remove_liquidity_event_2 = Event::dex(crate::Event::RemoveLiquidity(
				ALICE,
				JUSD,
				1_000_000_000_000,
				DNAR,
				200_000_000_000,
				2_000_000_000_000,
			));
			assert!(System::events()
				.iter()
				.any(|record| record.event == remove_liquidity_event_2));

			assert_eq!(SettinDex::get_liquidity(JUSD, DNAR), (0, 0));
			assert_eq!(Tokens::free_balance(JUSD, &SettinDex::account_id()), 0);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 0);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(JUSD, &ALICE), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &ALICE), 1_000_000_000_000_000_000);

			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(BOB),
				JUSD,
				DNAR,
				5_000_000_000_000,
				1_000_000_000_000,
				true
			));
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				10_000_000_000_000
			);
			assert_ok!(SettinDex::remove_liquidity(
				Origin::signed(BOB),
				JUSD,
				DNAR,
				2_000_000_000_000,
				true,
			));
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &BOB),
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

			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(ALICE),
				JUSD,
				DNAR,
				500_000_000_000_000,
				100_000_000_000_000,
				false,
			));
			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(ALICE),
				JUSD,
				JCHF,
				100_000_000_000_000,
				10_000_000_000,
				false,
			));

			assert_eq!(
				SettinDex::get_liquidity(JUSD, DNAR),
				(500_000_000_000_000, 100_000_000_000_000)
			);
			assert_eq!(
				SettinDex::get_liquidity(JUSD, JCHF),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(JUSD, &SettinDex::account_id()),
				600_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 100_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &SettinDex::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				SettinDex::do_swap_with_exact_supply(
					&BOB,
					&[DNAR, JUSD],
					100_000_000_000_000,
					250_000_000_000_000,
					None
				),
				Error::<Runtime>::InsufficientTargetAmount
			);
			assert_noop!(
				SettinDex::do_swap_with_exact_supply(
					&BOB,
					&[DNAR, JUSD],
					100_000_000_000_000,
					0,
					Ratio::checked_from_rational(10, 100)
				),
				Error::<Runtime>::ExceedPriceImpactLimit,
			);
			assert_noop!(
				SettinDex::do_swap_with_exact_supply(&BOB, &[DNAR, JUSD, JCHF, DNAR], 100_000_000_000_000, 0, None),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				SettinDex::do_swap_with_exact_supply(&BOB, &[DNAR, DNAR], 100_000_000_000_000, 0, None),
				Error::<Runtime>::MustBeEnabled,
			);

			assert_ok!(SettinDex::do_swap_with_exact_supply(
				&BOB,
				&[DNAR, JUSD],
				100_000_000_000_000,
				200_000_000_000_000,
				None
			));
			let swap_event_1 = Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, JUSD],
				100_000_000_000_000,
				248_743_718_592_964,
			));
			assert!(System::events().iter().any(|record| record.event == swap_event_1));

			assert_eq!(
				SettinDex::get_liquidity(JUSD, DNAR),
				(251_256_281_407_036, 200_000_000_000_000)
			);
			assert_eq!(
				SettinDex::get_liquidity(JUSD, JCHF),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(JUSD, &SettinDex::account_id()),
				351_256_281_407_036
			);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 200_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &SettinDex::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 1_000_248_743_718_592_964);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_900_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(SettinDex::do_swap_with_exact_supply(
				&BOB,
				&[DNAR, JUSD, JCHF],
				200_000_000_000_000,
				1,
				None
			));
			let swap_event_2 = Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, JUSD, JCHF],
				200_000_000_000_000,
				5_530_663_837,
			));
			assert!(System::events().iter().any(|record| record.event == swap_event_2));

			assert_eq!(
				SettinDex::get_liquidity(JUSD, DNAR),
				(126_259_437_892_983, 400_000_000_000_000)
			);
			assert_eq!(
				SettinDex::get_liquidity(JUSD, JCHF),
				(224_996_843_514_053, 4_469_336_163)
			);
			assert_eq!(
				Tokens::free_balance(JUSD, &SettinDex::account_id()),
				351_256_281_407_036
			);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 400_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &SettinDex::account_id()), 4_469_336_163);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 1_000_248_743_718_592_964);
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

			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(ALICE),
				JUSD,
				DNAR,
				500_000_000_000_000,
				100_000_000_000_000,
				false,
			));
			assert_ok!(SettinDex::add_liquidity(
				Origin::signed(ALICE),
				JUSD,
				JCHF,
				100_000_000_000_000,
				10_000_000_000,
				false,
			));

			assert_eq!(
				SettinDex::get_liquidity(JUSD, DNAR),
				(500_000_000_000_000, 100_000_000_000_000)
			);
			assert_eq!(
				SettinDex::get_liquidity(JUSD, JCHF),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(JUSD, &SettinDex::account_id()),
				600_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 100_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &SettinDex::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(JCHF, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				SettinDex::do_swap_with_exact_target(
					&BOB,
					&[DNAR, JUSD],
					250_000_000_000_000,
					100_000_000_000_000,
					None
				),
				Error::<Runtime>::ExcessiveSupplyAmount
			);
			assert_noop!(
				SettinDex::do_swap_with_exact_target(
					&BOB,
					&[DNAR, JUSD],
					250_000_000_000_000,
					200_000_000_000_000,
					Ratio::checked_from_rational(10, 100)
				),
				Error::<Runtime>::ExceedPriceImpactLimit,
			);
			assert_noop!(
				SettinDex::do_swap_with_exact_target(
					&BOB,
					&[DNAR, JUSD, JCHF, DNAR],
					250_000_000_000_000,
					200_000_000_000_000,
					None
				),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				SettinDex::do_swap_with_exact_target(&BOB, &[DNAR, DNAR], 250_000_000_000_000, 200_000_000_000_000, None),
				Error::<Runtime>::MustBeEnabled,
			);

			assert_ok!(SettinDex::do_swap_with_exact_target(
				&BOB,
				&[DNAR, JUSD],
				250_000_000_000_000,
				200_000_000_000_000,
				None
			));
			let swap_event_1 = Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, JUSD],
				101_010_101_010_102,
				250_000_000_000_000,
			));
			assert!(System::events().iter().any(|record| record.event == swap_event_1));

			assert_eq!(
				SettinDex::get_liquidity(JUSD, DNAR),
				(250_000_000_000_000, 201_010_101_010_102)
			);
			assert_eq!(
				SettinDex::get_liquidity(JUSD, JCHF),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(JUSD, &SettinDex::account_id()),
				350_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 201_010_101_010_102);
			assert_eq!(Tokens::free_balance(JCHF, &SettinDex::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 1_000_250_000_000_000_000);
			assert_eq!(Tokens::free_balance(DNAR, &BOB), 999_898_989_898_989_898);
			assert_eq!(Tokens::free_balance(JCHF, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(SettinDex::do_swap_with_exact_target(
				&BOB,
				&[DNAR, JUSD, JCHF],
				5_000_000_000,
				2_000_000_000_000_000,
				None
			));
			let swap_event_2 = Event::dex(crate::Event::Swap(
				BOB,
				vec![DNAR, JUSD, JCHF],
				137_654_580_386_993,
				5_000_000_000,
			));
			assert!(System::events().iter().any(|record| record.event == swap_event_2));

			assert_eq!(
				SettinDex::get_liquidity(JUSD, DNAR),
				(148_989_898_989_898, 338_664_681_397_095)
			);
			assert_eq!(
				SettinDex::get_liquidity(JUSD, JCHF),
				(201_010_101_010_102, 5_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(JUSD, &SettinDex::account_id()),
				350_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 338_664_681_397_095);
			assert_eq!(Tokens::free_balance(JCHF, &SettinDex::account_id()), 5_000_000_000);
			assert_eq!(Tokens::free_balance(JUSD, &BOB), 1_000_250_000_000_000_000);
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

			assert_eq!(SettinDex::get_liquidity(JUSD, DNAR), (1000000, 2000000));
			assert_eq!(Tokens::free_balance(JUSD, &SettinDex::account_id()), 2000000);
			assert_eq!(Tokens::free_balance(DNAR, &SettinDex::account_id()), 3000000);
			assert_eq!(
				Tokens::free_balance(JUSD_DNAR_PAIR.get_dex_share_currency_id().unwrap(), &ALICE),
				4000000
			);
		});
}
