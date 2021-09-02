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
	SETUSDBTCPair, SETUSDDRAMPair, DexModule, Event, ExtBuilder, ListingOrigin, Origin, Runtime, System, Tokens, DNAR, ALICE,
	SETUSD, BOB, BTC, DRAM,
};
use orml_traits::MultiReservableCurrency;
use sp_runtime::traits::BadOrigin;

#[test]
fn list_provisioning_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			DexModule::list_provisioning(
				Origin::signed(ALICE),
				SETUSD,
				DRAM,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			BadOrigin
		);

		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Disabled
		);
		assert_ok!(DexModule::list_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (2_000_000_000_000u128, 5_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);
		System::assert_last_event(Event::DexModule(crate::Event::ListProvisioning(SETUSDDRAMPair::get())));

		assert_noop!(
			DexModule::list_provisioning(
				Origin::signed(ListingOrigin::get()),
				SETUSD,
				SETUSD,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			Error::<Runtime>::InvalidCurrencyId
		);

		assert_noop!(
			DexModule::list_provisioning(
				Origin::signed(ListingOrigin::get()),
				SETUSD,
				DRAM,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			Error::<Runtime>::MustBeDisabled
		);
	});
}

#[test]
fn update_provisioning_parameters_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			DexModule::update_provisioning_parameters(
				Origin::signed(ALICE),
				SETUSD,
				DRAM,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			BadOrigin
		);

		assert_noop!(
			DexModule::update_provisioning_parameters(
				Origin::signed(ListingOrigin::get()),
				SETUSD,
				DRAM,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			Error::<Runtime>::MustBeProvisioning
		);

		assert_ok!(DexModule::list_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (2_000_000_000_000u128, 5_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);

		assert_ok!(DexModule::update_provisioning_parameters(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM,
			2_000_000_000_000u128,
			0,
			3_000_000_000_000u128,
			2_000_000_000_000u128,
			50,
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (0, 2_000_000_000_000u128),
				target_provision: (2_000_000_000_000u128, 3_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 50,
			})
		);
	});
}

#[test]
fn enable_diabled_trading_pair_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			DexModule::enable_trading_pair(Origin::signed(ALICE), SETUSD, DRAM),
			BadOrigin
		);

		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Disabled
		);
		assert_ok!(DexModule::enable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Enabled
		);
		System::assert_last_event(Event::DexModule(crate::Event::EnableTradingPair(SETUSDDRAMPair::get())));

		assert_noop!(
			DexModule::enable_trading_pair(Origin::signed(ListingOrigin::get()), DRAM, SETUSD),
			Error::<Runtime>::AlreadyEnabled
		);
	});
}

#[test]
fn enable_provisioning_without_provision_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(DexModule::list_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_ok!(DexModule::list_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			BTC,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_ok!(DexModule::add_provision(
			Origin::signed(ALICE),
			SETUSD,
			BTC,
			1_000_000_000_000u128,
			1_000_000_000_000u128
		));

		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (2_000_000_000_000u128, 5_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);
		assert_ok!(DexModule::enable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Enabled
		);
		System::assert_last_event(Event::DexModule(crate::Event::EnableTradingPair(SETUSDDRAMPair::get())));

		assert_noop!(
			DexModule::enable_trading_pair(Origin::signed(ListingOrigin::get()), SETUSD, BTC),
			Error::<Runtime>::StillProvisioning
		);
	});
}

#[test]
fn end_provisioning_trading_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(DexModule::list_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (2_000_000_000_000u128, 5_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);

		assert_ok!(DexModule::list_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			BTC,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_ok!(DexModule::add_provision(
			Origin::signed(ALICE),
			SETUSD,
			BTC,
			1_000_000_000_000u128,
			2_000_000_000_000u128
		));

		assert_noop!(
			DexModule::end_provisioning(Origin::signed(ListingOrigin::get()), SETUSD, BTC),
			Error::<Runtime>::UnqualifiedProvision
		);
		System::set_block_number(10);

		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDBTCPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (1_000_000_000_000u128, 2_000_000_000_000u128),
				not_before: 10,
			})
		);
		assert_eq!(
			DexModule::initial_share_exchange_rates(SETUSDBTCPair::get()),
			Default::default()
		);
		assert_eq!(DexModule::liquidity_pool(SETUSDBTCPair::get()), (0, 0));
		assert_eq!(Tokens::total_issuance(SETUSDBTCPair::get().dex_share_currency_id()), 0);
		assert_eq!(
			Tokens::free_balance(SETUSDBTCPair::get().dex_share_currency_id(), &DexModule::account_id()),
			0
		);

		assert_ok!(DexModule::end_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			BTC
		));
		System::assert_last_event(Event::DexModule(crate::Event::ProvisioningToEnabled(
			SETUSDBTCPair::get(),
			1_000_000_000_000u128,
			2_000_000_000_000u128,
			2_000_000_000_000u128,
		)));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDBTCPair::get()),
			TradingPairStatus::<_, _>::Enabled
		);
		assert_eq!(
			DexModule::initial_share_exchange_rates(SETUSDBTCPair::get()),
			(ExchangeRate::one(), ExchangeRate::checked_from_rational(1, 2).unwrap())
		);
		assert_eq!(
			DexModule::liquidity_pool(SETUSDBTCPair::get()),
			(1_000_000_000_000u128, 2_000_000_000_000u128)
		);
		assert_eq!(
			Tokens::total_issuance(SETUSDBTCPair::get().dex_share_currency_id()),
			2_000_000_000_000u128
		);
		assert_eq!(
			Tokens::free_balance(SETUSDBTCPair::get().dex_share_currency_id(), &DexModule::account_id()),
			2_000_000_000_000u128
		);
	});
}

#[test]
fn disable_trading_pair_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(DexModule::enable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Enabled
		);

		assert_noop!(
			DexModule::disable_trading_pair(Origin::signed(ALICE), SETUSD, DRAM),
			BadOrigin
		);

		assert_ok!(DexModule::disable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Disabled
		);
		System::assert_last_event(Event::DexModule(crate::Event::DisableTradingPair(SETUSDDRAMPair::get())));

		assert_noop!(
			DexModule::disable_trading_pair(Origin::signed(ListingOrigin::get()), SETUSD, DRAM),
			Error::<Runtime>::MustBeEnabled
		);

		assert_ok!(DexModule::list_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			BTC,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_noop!(
			DexModule::disable_trading_pair(Origin::signed(ListingOrigin::get()), SETUSD, BTC),
			Error::<Runtime>::MustBeEnabled
		);
	});
}

#[test]
fn add_provision_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			DexModule::add_provision(
				Origin::signed(ALICE),
				SETUSD,
				DRAM,
				5_000_000_000_000u128,
				1_000_000_000_000u128,
			),
			Error::<Runtime>::MustBeProvisioning
		);

		assert_ok!(DexModule::list_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM,
			5_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000_000u128,
			1_000_000_000_000_000u128,
			10,
		));

		assert_noop!(
			DexModule::add_provision(
				Origin::signed(ALICE),
				SETUSD,
				DRAM,
				4_999_999_999_999u128,
				999_999_999_999u128,
			),
			Error::<Runtime>::InvalidContributionIncrement
		);

		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 5_000_000_000_000u128),
				target_provision: (1_000_000_000_000_000u128, 5_000_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);
		assert_eq!(DexModule::provisioning_pool(SETUSDDRAMPair::get(), ALICE), (0, 0));
		assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 1_000_000_000_000_000_000u128);
		assert_eq!(Tokens::free_balance(DRAM, &ALICE), 1_000_000_000_000_000_000u128);
		assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 0);
		assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 0);
		let alice_ref_count_0 = System::consumers(&ALICE);

		assert_ok!(DexModule::add_provision(
			Origin::signed(ALICE),
			SETUSD,
			DRAM,
			5_000_000_000_000u128,
			0,
		));
		assert_eq!(
			DexModule::trading_pair_statuses(SETUSDDRAMPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 5_000_000_000_000u128),
				target_provision: (1_000_000_000_000_000u128, 5_000_000_000_000_000u128),
				accumulated_provision: (0, 5_000_000_000_000u128),
				not_before: 10,
			})
		);
		assert_eq!(
			DexModule::provisioning_pool(SETUSDDRAMPair::get(), ALICE),
			(0, 5_000_000_000_000u128)
		);
		assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 999_995_000_000_000_000u128);
		assert_eq!(Tokens::free_balance(DRAM, &ALICE), 1_000_000_000_000_000_000u128);
		assert_eq!(
			Tokens::free_balance(SETUSD, &DexModule::account_id()),
			5_000_000_000_000u128
		);
		assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 0);
		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 + 1);
		System::assert_last_event(Event::DexModule(crate::Event::AddProvision(
			ALICE,
			DRAM,
			0,
			SETUSD,
			5_000_000_000_000u128,
		)));
	});
}

#[test]
fn claim_dex_share_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(DexModule::list_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM,
			5_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000_000u128,
			1_000_000_000_000_000u128,
			0,
		));

		assert_ok!(DexModule::add_provision(
			Origin::signed(ALICE),
			SETUSD,
			DRAM,
			1_000_000_000_000_000u128,
			200_000_000_000_000u128,
		));
		assert_ok!(DexModule::add_provision(
			Origin::signed(BOB),
			SETUSD,
			DRAM,
			4_000_000_000_000_000u128,
			800_000_000_000_000u128,
		));

		assert_noop!(
			DexModule::claim_dex_share(Origin::signed(ALICE), ALICE, SETUSD, DRAM),
			Error::<Runtime>::StillProvisioning
		);

		assert_ok!(DexModule::end_provisioning(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM
		));

		let lp_currency_id = SETUSDDRAMPair::get().dex_share_currency_id();

		assert_eq!(
			InitialShareExchangeRates::<Runtime>::contains_key(SETUSDDRAMPair::get()),
			true
		);
		assert_eq!(
			DexModule::initial_share_exchange_rates(SETUSDDRAMPair::get()),
			(ExchangeRate::one(), ExchangeRate::saturating_from_rational(1, 5))
		);
		assert_eq!(
			Tokens::free_balance(lp_currency_id, &DexModule::account_id()),
			2_000_000_000_000_000u128
		);
		assert_eq!(
			DexModule::provisioning_pool(SETUSDDRAMPair::get(), ALICE),
			(200_000_000_000_000u128, 1_000_000_000_000_000u128)
		);
		assert_eq!(
			DexModule::provisioning_pool(SETUSDDRAMPair::get(), BOB),
			(800_000_000_000_000u128, 4_000_000_000_000_000u128)
		);
		assert_eq!(Tokens::free_balance(lp_currency_id, &ALICE), 0);
		assert_eq!(Tokens::free_balance(lp_currency_id, &BOB), 0);

		let alice_ref_count_0 = System::consumers(&ALICE);
		let bob_ref_count_0 = System::consumers(&BOB);

		assert_ok!(DexModule::claim_dex_share(Origin::signed(ALICE), ALICE, SETUSD, DRAM));
		assert_eq!(
			Tokens::free_balance(lp_currency_id, &DexModule::account_id()),
			1_600_000_000_000_000u128
		);
		assert_eq!(DexModule::provisioning_pool(SETUSDDRAMPair::get(), ALICE), (0, 0));
		assert_eq!(Tokens::free_balance(lp_currency_id, &ALICE), 400_000_000_000_000u128);
		assert_eq!(System::consumers(&ALICE), alice_ref_count_0 - 1);
		assert_eq!(
			InitialShareExchangeRates::<Runtime>::contains_key(SETUSDDRAMPair::get()),
			true
		);

		assert_ok!(DexModule::disable_trading_pair(
			Origin::signed(ListingOrigin::get()),
			SETUSD,
			DRAM
		));
		assert_ok!(DexModule::claim_dex_share(Origin::signed(BOB), BOB, SETUSD, DRAM));
		assert_eq!(Tokens::free_balance(lp_currency_id, &DexModule::account_id()), 0);
		assert_eq!(DexModule::provisioning_pool(SETUSDDRAMPair::get(), BOB), (0, 0));
		assert_eq!(Tokens::free_balance(lp_currency_id, &BOB), 1_600_000_000_000_000u128);
		assert_eq!(System::consumers(&BOB), bob_ref_count_0 - 1);
		assert_eq!(
			InitialShareExchangeRates::<Runtime>::contains_key(SETUSDDRAMPair::get()),
			false
		);
	});
}

#[test]
fn get_liquidity_work() {
	ExtBuilder::default().build().execute_with(|| {
		LiquidityPool::<Runtime>::insert(SETUSDDRAMPair::get(), (1000, 20));
		assert_eq!(DexModule::liquidity_pool(SETUSDDRAMPair::get()), (1000, 20));
		assert_eq!(DexModule::get_liquidity(SETUSD, DRAM), (20, 1000));
		assert_eq!(DexModule::get_liquidity(DRAM, SETUSD), (1000, 20));
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
			LiquidityPool::<Runtime>::insert(SETUSDDRAMPair::get(), (50000, 10000));
			LiquidityPool::<Runtime>::insert(SETUSDBTCPair::get(), (100000, 10));
			assert_noop!(
				DexModule::get_target_amounts(&vec![DRAM], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				DexModule::get_target_amounts(&vec![DRAM, SETUSD, BTC, DRAM], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				DexModule::get_target_amounts(&vec![DRAM, SETUSD, DNAR], 10000),
				Error::<Runtime>::MustBeEnabled,
			);
			assert_eq!(
				DexModule::get_target_amounts(&vec![DRAM, SETUSD], 10000),
				Ok(vec![10000, 1652])
			);
			assert_eq!(
				DexModule::get_target_amounts(&vec![DRAM, SETUSD], 10000),
				Ok(vec![10000, 1652])
			);
			assert_noop!(
				DexModule::get_target_amounts(&vec![DRAM, SETUSD, BTC], 100),
				Error::<Runtime>::ZeroTargetAmount,
			);
			assert_noop!(
				DexModule::get_target_amounts(&vec![DRAM, BTC], 100),
				Error::<Runtime>::InsufficientLiquidity,
			);
		});
}

#[test]
fn calculate_amount_for_big_number_work() {
	ExtBuilder::default().build().execute_with(|| {
		LiquidityPool::<Runtime>::insert(
			SETUSDDRAMPair::get(),
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
			LiquidityPool::<Runtime>::insert(SETUSDDRAMPair::get(), (50000, 10000));
			LiquidityPool::<Runtime>::insert(SETUSDBTCPair::get(), (100000, 10));
			assert_noop!(
				DexModule::get_supply_amounts(&vec![DRAM], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				DexModule::get_supply_amounts(&vec![DRAM, SETUSD, BTC, DRAM], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				DexModule::get_supply_amounts(&vec![DRAM, SETUSD, DNAR], 10000),
				Error::<Runtime>::MustBeEnabled,
			);
			assert_noop!(
				DexModule::get_supply_amounts(&vec![DRAM, SETUSD, BTC], 10000),
				Error::<Runtime>::ZeroSupplyAmount,
			);
			assert_noop!(
				DexModule::get_supply_amounts(&vec![DRAM, BTC], 10000),
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
			LiquidityPool::<Runtime>::insert(SETUSDDRAMPair::get(), (50000, 10000));

			assert_eq!(DexModule::get_liquidity(SETUSD, DRAM), (10000, 50000));
			assert_ok!(DexModule::_swap(SETUSD, DRAM, 50000, 5000));
			assert_eq!(DexModule::get_liquidity(SETUSD, DRAM), (60000, 45000));
		});
}

#[test]
fn _swap_by_path_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(SETUSDDRAMPair::get(), (50000, 10000));
			LiquidityPool::<Runtime>::insert(SETUSDBTCPair::get(), (100000, 10));

			assert_eq!(DexModule::get_liquidity(SETUSD, DRAM), (10000, 50000));
			assert_eq!(DexModule::get_liquidity(SETUSD, BTC), (100000, 10));
			assert_ok!(DexModule::_swap_by_path(&vec![DRAM, SETUSD], &vec![10000, 1000]));
			assert_eq!(DexModule::get_liquidity(SETUSD, DRAM), (9000, 60000));
			assert_eq!(DexModule::get_liquidity(SETUSD, BTC), (100000, 10));
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
				DexModule::add_liquidity(Origin::signed(ALICE), DNAR, SETUSD, 100_000_000, 100_000_000, 0),
				Error::<Runtime>::MustBeEnabled
			);
			assert_noop!(
				DexModule::add_liquidity(Origin::signed(ALICE), SETUSD, DRAM, 0, 100_000_000, 0),
				Error::<Runtime>::InvalidLiquidityIncrement
			);

			assert_eq!(DexModule::get_liquidity(SETUSD, DRAM), (0, 0));
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 0);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 0);
			assert_eq!(
				Tokens::free_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &ALICE),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &ALICE), 1_000_000_000_000_000_000);

			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				DRAM,
				5_000_000_000_000,
				1_000_000_000_000,
				0
			));
			System::assert_last_event(Event::DexModule(crate::Event::AddLiquidity(
				ALICE,
				DRAM,
				1_000_000_000_000,
				SETUSD,
				5_000_000_000_000,
				2_000_000_000_000,
			)));
			assert_eq!(
				DexModule::get_liquidity(SETUSD, DRAM),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &ALICE),
				2_000_000_000_000
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &ALICE), 999_999_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				DexModule::add_liquidity(Origin::signed(BOB), SETUSD, DRAM, 4, 1, 0,),
				Error::<Runtime>::InvalidLiquidityIncrement,
			);

			assert_noop!(
				DexModule::add_liquidity(
					Origin::signed(BOB),
					SETUSD,
					DRAM,
					50_000_000_000_000,
					8_000_000_000_000,
					80_000_000_000_001,
				),
				Error::<Runtime>::UnacceptableShareIncrement
			);

			assert_noop!(
				DexModule::add_liquidity(
				Origin::signed(BOB),
				SETUSD,
				DRAM,
				50_000_000_000_000,
				8_000_000_000_000,
				80_000_000_000_000,
				),
				Error::<Runtime>::UnacceptableShareIncrement
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, DRAM),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &BOB), 1_000_000_000_000_000_000);
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
				DRAM,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
			));
			assert_noop!(
				DexModule::remove_liquidity(
					Origin::signed(ALICE),
					SETUSDDRAMPair::get().dex_share_currency_id(),
					DRAM,
					100_000_000,
					0,
					0,
				),
				Error::<Runtime>::InvalidCurrencyId
			);

			assert_eq!(
				DexModule::get_liquidity(SETUSD, DRAM),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &ALICE),
				2_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &ALICE), 999_999_000_000_000_000);

			System::assert_last_event(Event::DexModule(crate::Event::AddLiquidity(
				ALICE,
				DRAM,
				1_000_000_000_000,
				SETUSD,
				5_000_000_000_000,
				2_000_000_000_000,
			)));
			assert_eq!(
				DexModule::get_liquidity(SETUSD, DRAM),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &ALICE),
				2_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &ALICE), 999_999_000_000_000_000);

			assert_ok!(DexModule::remove_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				DRAM,
				2_000_000_000_000,
				0,
				0,
			));
			System::assert_last_event(Event::DexModule(crate::Event::RemoveLiquidity(
				ALICE,
				DRAM,
				1_000_000_000_000,
				SETUSD,
				5_000_000_000_000,
				2_000_000_000_000,
			)));
			assert_eq!(DexModule::get_liquidity(SETUSD, DRAM), (0, 0));
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 0);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 0);
			assert_eq!(
				Tokens::free_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(SETUSD, &ALICE), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &ALICE), 1_000_000_000_000_000_000);

			assert_ok!(DexModule::add_liquidity(
				Origin::signed(BOB),
				SETUSD,
				DRAM,
				5_000_000_000_000,
				1_000_000_000_000,
				0
			));
			assert_eq!(
				Tokens::free_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &BOB),
				2_000_000_000_000,
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_ok!(DexModule::remove_liquidity(
				Origin::signed(BOB),
				SETUSD,
				DRAM,
				2_000_000_000_000,
				0,
				0,
			));
			assert_eq!(
				Tokens::free_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &BOB),
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
				DRAM,
				500_000_000_000_000,
				100_000_000_000_000,
				0,
			));
			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				BTC,
				100_000_000_000_000,
				10_000_000_000,
				0,
			));

			assert_eq!(
				DexModule::get_liquidity(SETUSD, DRAM),
				(500_000_000_000_000, 100_000_000_000_000)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, BTC),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				600_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 100_000_000_000_000);
			assert_eq!(Tokens::free_balance(BTC, &DexModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(BTC, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				DexModule::do_swap_with_exact_supply(
					&BOB,
					&[DRAM, SETUSD],
					100_000_000_000_000,
					250_000_000_000_000,
				),
				Error::<Runtime>::InsufficientTargetAmount
			);
			assert_noop!(
				DexModule::do_swap_with_exact_supply(&BOB, &[DRAM, SETUSD, BTC, DRAM], 100_000_000_000_000, 0),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				DexModule::do_swap_with_exact_supply(&BOB, &[DRAM, DNAR], 100_000_000_000_000, 0),
				Error::<Runtime>::MustBeEnabled,
			);

			assert_ok!(DexModule::do_swap_with_exact_supply(
				&BOB,
				&[DRAM, SETUSD],
				100_000_000_000_000,
				200_000_000_000_000,
			));
			System::assert_last_event(Event::DexModule(crate::Event::Swap(
				BOB,
				vec![DRAM, SETUSD],
				100_000_000_000_000,
				249_373_433_583_959,
			)));
			assert_eq!(
				DexModule::get_liquidity(SETUSD, DRAM),
				(251_256_281_407_036, 200_000_000_000_000)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, BTC),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				351_256_281_407_036
			);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 200_000_000_000_000);
			assert_eq!(Tokens::free_balance(BTC, &DexModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_249_373_433_583_959);
			assert_eq!(Tokens::free_balance(DRAM, &BOB), 999_900_000_000_000_000);
			assert_eq!(Tokens::free_balance(BTC, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(DexModule::do_swap_with_exact_supply(
				&BOB,
				&[DRAM, SETUSD, BTC],
				200_000_000_000_000,
				1,
			));
			System::assert_last_event(Event::DexModule(crate::Event::Swap(
				BOB,
				vec![DRAM, SETUSD, BTC],
				200_000_000_000_000,
				5_530_663_837,
			)));
			assert_eq!(
				DexModule::get_liquidity(SETUSD, DRAM),
				(126_259_437_892_983, 400_000_000_000_000)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, BTC),
				(224_996_843_514_053, 4_469_336_163)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				351_256_281_407_036
			);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 400_000_000_000_000);
			assert_eq!(Tokens::free_balance(BTC, &DexModule::account_id()), 4_469_336_163);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_249_373_433_583_959);
			assert_eq!(Tokens::free_balance(DRAM, &BOB), 999_700_000_000_000_000);
			assert_eq!(Tokens::free_balance(BTC, &BOB), 1_000_000_005_530_663_837);
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
				DRAM,
				500_000_000_000_000,
				100_000_000_000_000,
				0,
			));
			assert_ok!(DexModule::add_liquidity(
				Origin::signed(ALICE),
				SETUSD,
				BTC,
				100_000_000_000_000,
				10_000_000_000,
				0,
			));

			assert_eq!(
				DexModule::get_liquidity(SETUSD, DRAM),
				(500_000_000_000_000, 100_000_000_000_000)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, BTC),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				600_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 100_000_000_000_000);
			assert_eq!(Tokens::free_balance(BTC, &DexModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(BTC, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				DexModule::do_swap_with_exact_target(
					&BOB,
					&[DRAM, SETUSD],
					250_000_000_000_000,
					100_000_000_000_000,
				),
				Error::<Runtime>::ExcessiveSupplyAmount
			);
			assert_noop!(
				DexModule::do_swap_with_exact_target(
					&BOB,
					&[DRAM, SETUSD, BTC, DRAM],
					250_000_000_000_000,
					200_000_000_000_000,
				),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				DexModule::do_swap_with_exact_target(&BOB, &[DRAM, DNAR], 250_000_000_000_000, 200_000_000_000_000),
				Error::<Runtime>::MustBeEnabled,
			);

			assert_ok!(DexModule::do_swap_with_exact_target(
				&BOB,
				&[DRAM, SETUSD],
				250_000_000_000_000,
				200_000_000_000_000,
			));
			System::assert_last_event(Event::DexModule(crate::Event::Swap(
				BOB,
				vec![DRAM, SETUSD],
				100_502_512_562_815,
				250_000_000_000_000,
			)));
			assert_eq!(
				DexModule::get_liquidity(SETUSD, DRAM),
				(250_000_000_000_001, 200_502_512_562_815)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, BTC),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				350_000_000_000_001
			);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 200_502_512_562_815);
			assert_eq!(Tokens::free_balance(BTC, &DexModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_250_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &BOB), 999_899_497_487_437_185);
			assert_eq!(Tokens::free_balance(BTC, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(DexModule::do_swap_with_exact_target(
				&BOB,
				&[DRAM, SETUSD, BTC],
				5_000_000_000,
				2_000_000_000_000_000,
			));
			System::assert_last_event(Event::DexModule(crate::Event::Swap(
				BOB,
				vec![DRAM, SETUSD, BTC],
				137_306_976_588_131,
				5_000_000_000,
			)));
			assert_eq!(
				DexModule::get_liquidity(SETUSD, DRAM),
				(148_989_898_989_899, 337_809_489_150_946)
			);
			assert_eq!(
				DexModule::get_liquidity(SETUSD, BTC),
				(201_010_101_010_102, 5_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(SETUSD, &DexModule::account_id()),
				350_000_000_000_001
			);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 337_809_489_150_946);
			assert_eq!(Tokens::free_balance(BTC, &DexModule::account_id()), 5_000_000_000);
			assert_eq!(Tokens::free_balance(SETUSD, &BOB), 1_000_250_000_000_000_000);
			assert_eq!(Tokens::free_balance(DRAM, &BOB), 999_762_190_510_849_054);
			assert_eq!(Tokens::free_balance(BTC, &BOB), 1_000_000_005_000_000_000);
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

			assert_eq!(DexModule::get_liquidity(SETUSD, DRAM), (2000000, 1000000));
			assert_eq!(Tokens::free_balance(SETUSD, &DexModule::account_id()), 3000000);
			assert_eq!(Tokens::free_balance(DRAM, &DexModule::account_id()), 2000000);
			assert_eq!(
				Tokens::free_balance(SETUSDDRAMPair::get().dex_share_currency_id(), &ALICE),
				2000000
			);
		});
}
