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

//! Unit tests for the dex module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{
	SEEJointSwap, USSDWBTCPair, USSDEDFPair, USSDJointSwap, EDFWBTCPair, EdfisSwapModule, ExtBuilder, ListingOrigin, Runtime,
	RuntimeEvent, RuntimeOrigin, System, Tokens, SEE, ALICE, USSD, USSD_EDF_POOL_RECORD, BOB, WBTC, CAROL, EDF,
};
use module_support::{Swap, SwapError};
use orml_traits::MultiReservableCurrency;
use sp_core::H160;
use sp_runtime::traits::BadOrigin;
use std::str::FromStr;

#[test]
fn list_provisioning_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			EdfisSwapModule::list_provisioning(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			BadOrigin
		);

		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Disabled
		);
		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);
		System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::ListProvisioning {
			trading_pair: USSDEDFPair::get(),
		}));

		assert_noop!(
			EdfisSwapModule::list_provisioning(
				RuntimeOrigin::signed(ListingOrigin::get()),
				USSD,
				USSD,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			Error::<Runtime>::InvalidCurrencyId
		);

		assert_noop!(
			EdfisSwapModule::list_provisioning(
				RuntimeOrigin::signed(ListingOrigin::get()),
				USSD,
				EDF,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			Error::<Runtime>::MustBeDisabled
		);

		assert_noop!(
			EdfisSwapModule::list_provisioning(
				RuntimeOrigin::signed(ListingOrigin::get()),
				CurrencyId::ForeignAsset(0),
				USSD,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			Error::<Runtime>::AssetUnregistered
		);
		assert_noop!(
			EdfisSwapModule::list_provisioning(
				RuntimeOrigin::signed(ListingOrigin::get()),
				USSD,
				CurrencyId::ForeignAsset(0),
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			Error::<Runtime>::AssetUnregistered
		);
	});
}

#[test]
fn update_provisioning_parameters_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			EdfisSwapModule::update_provisioning_parameters(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			BadOrigin
		);

		assert_noop!(
			EdfisSwapModule::update_provisioning_parameters(
				RuntimeOrigin::signed(ListingOrigin::get()),
				USSD,
				EDF,
				1_000_000_000_000u128,
				1_000_000_000_000u128,
				5_000_000_000_000u128,
				2_000_000_000_000u128,
				10,
			),
			Error::<Runtime>::MustBeProvisioning
		);

		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);

		assert_ok!(EdfisSwapModule::update_provisioning_parameters(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF,
			2_000_000_000_000u128,
			0,
			3_000_000_000_000u128,
			2_000_000_000_000u128,
			50,
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (2_000_000_000_000u128, 0),
				target_provision: (3_000_000_000_000u128, 2_000_000_000_000u128),
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
			EdfisSwapModule::enable_trading_pair(RuntimeOrigin::signed(ALICE), USSD, EDF),
			BadOrigin
		);

		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Disabled
		);
		assert_ok!(EdfisSwapModule::enable_trading_pair(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Enabled
		);
		System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::EnableTradingPair {
			trading_pair: USSDEDFPair::get(),
		}));

		assert_noop!(
			EdfisSwapModule::enable_trading_pair(RuntimeOrigin::signed(ListingOrigin::get()), EDF, USSD),
			Error::<Runtime>::AlreadyEnabled
		);
	});
}

#[test]
fn enable_provisioning_without_provision_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			WBTC,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_ok!(EdfisSwapModule::add_provision(
			RuntimeOrigin::signed(ALICE),
			USSD,
			WBTC,
			1_000_000_000_000u128,
			1_000_000_000_000u128
		));

		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);
		assert_ok!(EdfisSwapModule::enable_trading_pair(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Enabled
		);
		System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::EnableTradingPair {
			trading_pair: USSDEDFPair::get(),
		}));

		assert_noop!(
			EdfisSwapModule::enable_trading_pair(RuntimeOrigin::signed(ListingOrigin::get()), USSD, WBTC),
			Error::<Runtime>::StillProvisioning
		);
	});
}

#[test]
fn end_provisioning_trading_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);

		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			WBTC,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_ok!(EdfisSwapModule::add_provision(
			RuntimeOrigin::signed(ALICE),
			USSD,
			WBTC,
			1_000_000_000_000u128,
			2_000_000_000_000u128
		));

		assert_noop!(
			EdfisSwapModule::end_provisioning(RuntimeOrigin::signed(ListingOrigin::get()), USSD, WBTC),
			Error::<Runtime>::UnqualifiedProvision
		);
		System::set_block_number(10);

		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDWBTCPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (1_000_000_000_000u128, 2_000_000_000_000u128),
				not_before: 10,
			})
		);
		assert_eq!(
			EdfisSwapModule::initial_share_exchange_rates(USSDWBTCPair::get()),
			Default::default()
		);
		assert_eq!(EdfisSwapModule::liquidity_pool(USSDWBTCPair::get()), (0, 0));
		assert_eq!(Tokens::total_issuance(USSDWBTCPair::get().dex_share_currency_id()), 0);
		assert_eq!(
			Tokens::free_balance(USSDWBTCPair::get().dex_share_currency_id(), &EdfisSwapModule::account_id()),
			0
		);

		assert_ok!(EdfisSwapModule::end_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			WBTC
		));
		System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::ProvisioningToEnabled {
			trading_pair: USSDWBTCPair::get(),
			pool_0: 1_000_000_000_000u128,
			pool_1: 2_000_000_000_000u128,
			share_amount: 2_000_000_000_000u128,
		}));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDWBTCPair::get()),
			TradingPairStatus::<_, _>::Enabled
		);
		assert_eq!(
			EdfisSwapModule::initial_share_exchange_rates(USSDWBTCPair::get()),
			(ExchangeRate::one(), ExchangeRate::checked_from_rational(1, 2).unwrap())
		);
		assert_eq!(
			EdfisSwapModule::liquidity_pool(USSDWBTCPair::get()),
			(1_000_000_000_000u128, 2_000_000_000_000u128)
		);
		assert_eq!(
			Tokens::total_issuance(USSDWBTCPair::get().dex_share_currency_id()),
			2_000_000_000_000u128
		);
		assert_eq!(
			Tokens::free_balance(USSDWBTCPair::get().dex_share_currency_id(), &EdfisSwapModule::account_id()),
			2_000_000_000_000u128
		);
	});
}

#[test]
fn abort_provisioning_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			EdfisSwapModule::abort_provisioning(RuntimeOrigin::signed(ALICE), USSD, EDF),
			Error::<Runtime>::MustBeProvisioning
		);

		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			1000,
		));
		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			WBTC,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			1000,
		));

		assert_ok!(EdfisSwapModule::add_provision(
			RuntimeOrigin::signed(ALICE),
			USSD,
			EDF,
			1_000_000_000_000u128,
			1_000_000_000_000u128
		));
		assert_ok!(EdfisSwapModule::add_provision(
			RuntimeOrigin::signed(BOB),
			USSD,
			WBTC,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
		));

		// not expired, nothing happened.
		System::set_block_number(2000);
		assert_ok!(EdfisSwapModule::abort_provisioning(RuntimeOrigin::signed(ALICE), USSD, EDF));
		assert_ok!(EdfisSwapModule::abort_provisioning(RuntimeOrigin::signed(ALICE), USSD, WBTC));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (1_000_000_000_000u128, 1_000_000_000_000u128),
				not_before: 1000,
			})
		);
		assert_eq!(
			EdfisSwapModule::initial_share_exchange_rates(USSDEDFPair::get()),
			Default::default()
		);
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDWBTCPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				not_before: 1000,
			})
		);
		assert_eq!(
			EdfisSwapModule::initial_share_exchange_rates(USSDWBTCPair::get()),
			Default::default()
		);

		// both expired, the provision for USSD-EDF could be aborted, the provision for USSD-WBTC
		// couldn't be aborted because it's already met the target.
		System::set_block_number(3001);
		assert_ok!(EdfisSwapModule::abort_provisioning(RuntimeOrigin::signed(ALICE), USSD, EDF));
		System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::ProvisioningAborted {
			trading_pair: USSDEDFPair::get(),
			accumulated_provision_0: 1_000_000_000_000u128,
			accumulated_provision_1: 1_000_000_000_000u128,
		}));

		assert_ok!(EdfisSwapModule::abort_provisioning(RuntimeOrigin::signed(ALICE), USSD, WBTC));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Disabled
		);
		assert_eq!(
			EdfisSwapModule::initial_share_exchange_rates(USSDEDFPair::get()),
			Default::default()
		);
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDWBTCPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				not_before: 1000,
			})
		);
		assert_eq!(
			EdfisSwapModule::initial_share_exchange_rates(USSDWBTCPair::get()),
			Default::default()
		);
	});
}

#[test]
fn refund_provision_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF,
			1_000_000_000_000_000u128,
			1_000_000_000_000_000u128,
			5_000_000_000_000_000_000u128,
			4_000_000_000_000_000_000u128,
			1000,
		));
		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			WBTC,
			1_000_000_000_000_000u128,
			1_000_000_000_000_000u128,
			100_000_000_000_000_000u128,
			100_000_000_000_000_000u128,
			1000,
		));

		assert_ok!(EdfisSwapModule::add_provision(
			RuntimeOrigin::signed(ALICE),
			USSD,
			EDF,
			1_000_000_000_000_000_000u128,
			1_000_000_000_000_000_000u128
		));
		assert_ok!(EdfisSwapModule::add_provision(
			RuntimeOrigin::signed(BOB),
			USSD,
			EDF,
			0,
			600_000_000_000_000_000u128,
		));
		assert_ok!(EdfisSwapModule::add_provision(
			RuntimeOrigin::signed(BOB),
			USSD,
			WBTC,
			100_000_000_000_000_000u128,
			100_000_000_000_000_000u128,
		));

		assert_noop!(
			EdfisSwapModule::refund_provision(RuntimeOrigin::signed(ALICE), ALICE, USSD, EDF),
			Error::<Runtime>::MustBeDisabled
		);

		// abort provisioning of USSD-EDF
		System::set_block_number(3001);
		assert_ok!(EdfisSwapModule::abort_provisioning(RuntimeOrigin::signed(ALICE), USSD, EDF));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Disabled
		);
		assert_eq!(
			EdfisSwapModule::initial_share_exchange_rates(USSDEDFPair::get()),
			Default::default()
		);

		assert_eq!(
			EdfisSwapModule::provisioning_pool(USSDEDFPair::get(), ALICE),
			(1_000_000_000_000_000_000u128, 1_000_000_000_000_000_000u128)
		);
		assert_eq!(
			EdfisSwapModule::provisioning_pool(USSDEDFPair::get(), BOB),
			(0, 600_000_000_000_000_000u128)
		);
		assert_eq!(
			Tokens::free_balance(USSD, &EdfisSwapModule::account_id()),
			1_100_000_000_000_000_000u128
		);
		assert_eq!(
			Tokens::free_balance(EDF, &EdfisSwapModule::account_id()),
			1_600_000_000_000_000_000u128
		);
		assert_eq!(Tokens::free_balance(USSD, &ALICE), 0);
		assert_eq!(Tokens::free_balance(EDF, &ALICE), 0);
		assert_eq!(Tokens::free_balance(USSD, &BOB), 900_000_000_000_000_000u128);
		assert_eq!(Tokens::free_balance(EDF, &BOB), 400_000_000_000_000_000u128);

		let alice_ref_count_0 = System::consumers(&ALICE);
		let bob_ref_count_0 = System::consumers(&BOB);

		assert_ok!(EdfisSwapModule::refund_provision(
			RuntimeOrigin::signed(ALICE),
			ALICE,
			USSD,
			EDF
		));
		System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::RefundProvision {
			who: ALICE,
			currency_0: USSD,
			contribution_0: 1_000_000_000_000_000_000u128,
			currency_1: EDF,
			contribution_1: 1_000_000_000_000_000_000u128,
		}));

		assert_eq!(EdfisSwapModule::provisioning_pool(USSDEDFPair::get(), ALICE), (0, 0));
		assert_eq!(
			Tokens::free_balance(USSD, &EdfisSwapModule::account_id()),
			100_000_000_000_000_000u128
		);
		assert_eq!(
			Tokens::free_balance(EDF, &EdfisSwapModule::account_id()),
			600_000_000_000_000_000u128
		);
		assert_eq!(Tokens::free_balance(USSD, &ALICE), 1_000_000_000_000_000_000u128);
		assert_eq!(Tokens::free_balance(EDF, &ALICE), 1_000_000_000_000_000_000u128);
		assert_eq!(System::consumers(&ALICE), alice_ref_count_0 - 1);

		assert_ok!(EdfisSwapModule::refund_provision(
			RuntimeOrigin::signed(ALICE),
			BOB,
			USSD,
			EDF
		));
		System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::RefundProvision {
			who: BOB,
			currency_0: USSD,
			contribution_0: 0,
			currency_1: EDF,
			contribution_1: 600_000_000_000_000_000u128,
		}));

		assert_eq!(EdfisSwapModule::provisioning_pool(USSDEDFPair::get(), BOB), (0, 0));
		assert_eq!(
			Tokens::free_balance(USSD, &EdfisSwapModule::account_id()),
			100_000_000_000_000_000u128
		);
		assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 0);
		assert_eq!(Tokens::free_balance(USSD, &BOB), 900_000_000_000_000_000u128);
		assert_eq!(Tokens::free_balance(EDF, &BOB), 1_000_000_000_000_000_000u128);
		assert_eq!(System::consumers(&BOB), bob_ref_count_0 - 1);

		// not allow refund if the provisioning has been ended before.
		assert_ok!(EdfisSwapModule::end_provisioning(RuntimeOrigin::signed(ALICE), USSD, WBTC));
		assert_ok!(EdfisSwapModule::disable_trading_pair(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			WBTC
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDWBTCPair::get()),
			TradingPairStatus::<_, _>::Disabled
		);
		assert_eq!(
			EdfisSwapModule::provisioning_pool(USSDWBTCPair::get(), BOB),
			(100_000_000_000_000_000u128, 100_000_000_000_000_000u128)
		);
		assert_noop!(
			EdfisSwapModule::refund_provision(RuntimeOrigin::signed(BOB), BOB, USSD, WBTC),
			Error::<Runtime>::NotAllowedRefund
		);
	});
}

#[test]
fn disable_trading_pair_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(EdfisSwapModule::enable_trading_pair(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Enabled
		);

		assert_noop!(
			EdfisSwapModule::disable_trading_pair(RuntimeOrigin::signed(ALICE), USSD, EDF),
			BadOrigin
		);

		assert_ok!(EdfisSwapModule::disable_trading_pair(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Disabled
		);
		System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::DisableTradingPair {
			trading_pair: USSDEDFPair::get(),
		}));

		assert_noop!(
			EdfisSwapModule::disable_trading_pair(RuntimeOrigin::signed(ListingOrigin::get()), USSD, EDF),
			Error::<Runtime>::MustBeEnabled
		);

		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			WBTC,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_noop!(
			EdfisSwapModule::disable_trading_pair(RuntimeOrigin::signed(ListingOrigin::get()), USSD, WBTC),
			Error::<Runtime>::MustBeEnabled
		);
	});
}

#[test]
fn on_liquidity_pool_updated_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				WBTC,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
				false,
			));
			assert_eq!(USSD_EDF_POOL_RECORD.with(|v| *v.borrow()), (0, 0));

			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
				false,
			));
			assert_eq!(
				USSD_EDF_POOL_RECORD.with(|v| *v.borrow()),
				(5000000000000, 1000000000000)
			);
		});
}

#[test]
fn add_provision_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			EdfisSwapModule::add_provision(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				5_000_000_000_000u128,
				1_000_000_000_000u128,
			),
			Error::<Runtime>::MustBeProvisioning
		);

		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF,
			5_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000_000u128,
			1_000_000_000_000_000u128,
			10,
		));

		assert_noop!(
			EdfisSwapModule::add_provision(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				4_999_999_999_999u128,
				999_999_999_999u128,
			),
			Error::<Runtime>::InvalidContributionIncrement
		);

		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);
		assert_eq!(EdfisSwapModule::provisioning_pool(USSDEDFPair::get(), ALICE), (0, 0));
		assert_eq!(Tokens::free_balance(USSD, &ALICE), 1_000_000_000_000_000_000u128);
		assert_eq!(Tokens::free_balance(EDF, &ALICE), 1_000_000_000_000_000_000u128);
		assert_eq!(Tokens::free_balance(USSD, &EdfisSwapModule::account_id()), 0);
		assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 0);
		let alice_ref_count_0 = System::consumers(&ALICE);

		assert_ok!(EdfisSwapModule::add_provision(
			RuntimeOrigin::signed(ALICE),
			USSD,
			EDF,
			5_000_000_000_000u128,
			0,
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (5_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
				accumulated_provision: (5_000_000_000_000u128, 0),
				not_before: 10,
			})
		);
		assert_eq!(
			EdfisSwapModule::provisioning_pool(USSDEDFPair::get(), ALICE),
			(5_000_000_000_000u128, 0)
		);
		assert_eq!(Tokens::free_balance(USSD, &ALICE), 999_995_000_000_000_000u128);
		assert_eq!(Tokens::free_balance(EDF, &ALICE), 1_000_000_000_000_000_000u128);
		assert_eq!(
			Tokens::free_balance(USSD, &EdfisSwapModule::account_id()),
			5_000_000_000_000u128
		);
		assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 0);
		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 + 1);
		System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::AddProvision {
			who: ALICE,
			currency_0: USSD,
			contribution_0: 5_000_000_000_000u128,
			currency_1: EDF,
			contribution_1: 0,
		}));
	});
}

#[test]
fn claim_dex_share_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF,
			5_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000_000u128,
			1_000_000_000_000_000u128,
			0,
		));

		assert_ok!(EdfisSwapModule::add_provision(
			RuntimeOrigin::signed(ALICE),
			USSD,
			EDF,
			1_000_000_000_000_000u128,
			200_000_000_000_000u128,
		));
		assert_ok!(EdfisSwapModule::add_provision(
			RuntimeOrigin::signed(BOB),
			USSD,
			EDF,
			4_000_000_000_000_000u128,
			800_000_000_000_000u128,
		));

		assert_noop!(
			EdfisSwapModule::claim_dex_share(RuntimeOrigin::signed(ALICE), ALICE, USSD, EDF),
			Error::<Runtime>::StillProvisioning
		);

		assert_ok!(EdfisSwapModule::end_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF
		));

		let lp_currency_id = USSDEDFPair::get().dex_share_currency_id();

		assert!(InitialShareExchangeRates::<Runtime>::contains_key(USSDEDFPair::get()),);
		assert_eq!(
			EdfisSwapModule::initial_share_exchange_rates(USSDEDFPair::get()),
			(ExchangeRate::one(), ExchangeRate::saturating_from_rational(5, 1))
		);
		assert_eq!(
			Tokens::free_balance(lp_currency_id, &EdfisSwapModule::account_id()),
			10_000_000_000_000_000u128
		);
		assert_eq!(
			EdfisSwapModule::provisioning_pool(USSDEDFPair::get(), ALICE),
			(1_000_000_000_000_000u128, 200_000_000_000_000u128)
		);
		assert_eq!(
			EdfisSwapModule::provisioning_pool(USSDEDFPair::get(), BOB),
			(4_000_000_000_000_000u128, 800_000_000_000_000u128)
		);
		assert_eq!(Tokens::free_balance(lp_currency_id, &ALICE), 0);
		assert_eq!(Tokens::free_balance(lp_currency_id, &BOB), 0);

		let alice_ref_count_0 = System::consumers(&ALICE);
		let bob_ref_count_0 = System::consumers(&BOB);

		assert_ok!(EdfisSwapModule::claim_dex_share(
			RuntimeOrigin::signed(ALICE),
			ALICE,
			USSD,
			EDF
		));
		assert_eq!(
			Tokens::free_balance(lp_currency_id, &EdfisSwapModule::account_id()),
			8_000_000_000_000_000u128
		);
		assert_eq!(EdfisSwapModule::provisioning_pool(USSDEDFPair::get(), ALICE), (0, 0));
		assert_eq!(Tokens::free_balance(lp_currency_id, &ALICE), 2_000_000_000_000_000u128);
		assert_eq!(System::consumers(&ALICE), alice_ref_count_0 - 1);
		assert!(InitialShareExchangeRates::<Runtime>::contains_key(USSDEDFPair::get()),);

		assert_ok!(EdfisSwapModule::disable_trading_pair(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF
		));
		assert_ok!(EdfisSwapModule::claim_dex_share(RuntimeOrigin::signed(BOB), BOB, USSD, EDF));
		assert_eq!(Tokens::free_balance(lp_currency_id, &EdfisSwapModule::account_id()), 0);
		assert_eq!(EdfisSwapModule::provisioning_pool(USSDEDFPair::get(), BOB), (0, 0));
		assert_eq!(Tokens::free_balance(lp_currency_id, &BOB), 8_000_000_000_000_000u128);
		assert_eq!(System::consumers(&BOB), bob_ref_count_0 - 1);
		assert!(!InitialShareExchangeRates::<Runtime>::contains_key(USSDEDFPair::get()),);
	});
}

#[test]
fn get_liquidity_work() {
	ExtBuilder::default().build().execute_with(|| {
		LiquidityPool::<Runtime>::insert(USSDEDFPair::get(), (1000, 20));
		assert_eq!(EdfisSwapModule::liquidity_pool(USSDEDFPair::get()), (1000, 20));
		assert_eq!(EdfisSwapModule::get_liquidity(USSD, EDF), (1000, 20));
		assert_eq!(EdfisSwapModule::get_liquidity(EDF, USSD), (20, 1000));
	});
}

#[test]
fn get_target_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(EdfisSwapModule::get_target_amount(10000, 0, 1000), 0);
		assert_eq!(EdfisSwapModule::get_target_amount(0, 20000, 1000), 0);
		assert_eq!(EdfisSwapModule::get_target_amount(10000, 20000, 0), 0);
		assert_eq!(EdfisSwapModule::get_target_amount(10000, 1, 1000000), 0);
		assert_eq!(EdfisSwapModule::get_target_amount(10000, 20000, 10000), 9949);
		assert_eq!(EdfisSwapModule::get_target_amount(10000, 20000, 1000), 1801);
	});
}

#[test]
fn get_supply_amount_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(EdfisSwapModule::get_supply_amount(10000, 0, 1000), 0);
		assert_eq!(EdfisSwapModule::get_supply_amount(0, 20000, 1000), 0);
		assert_eq!(EdfisSwapModule::get_supply_amount(10000, 20000, 0), 0);
		assert_eq!(EdfisSwapModule::get_supply_amount(10000, 1, 1), 0);
		assert_eq!(EdfisSwapModule::get_supply_amount(10000, 20000, 9949), 9999);
		assert_eq!(EdfisSwapModule::get_target_amount(10000, 20000, 9999), 9949);
		assert_eq!(EdfisSwapModule::get_supply_amount(10000, 20000, 1801), 1000);
		assert_eq!(EdfisSwapModule::get_target_amount(10000, 20000, 1000), 1801);
	});
}

#[test]
fn get_target_amounts_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(USSDEDFPair::get(), (50000, 10000));
			LiquidityPool::<Runtime>::insert(USSDWBTCPair::get(), (100000, 10));
			assert_noop!(
				EdfisSwapModule::get_target_amounts(&[EDF], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				EdfisSwapModule::get_target_amounts(&[EDF, USSD, WBTC, EDF], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				EdfisSwapModule::get_target_amounts(&[EDF, EDF], 10000),
				Error::<Runtime>::InvalidTradingPath,
			);
			assert_noop!(
				EdfisSwapModule::get_target_amounts(&[EDF, USSD, EDF], 10000),
				Error::<Runtime>::InvalidTradingPath,
			);
			assert_noop!(
				EdfisSwapModule::get_target_amounts(&[EDF, USSD, SEE], 10000),
				Error::<Runtime>::MustBeEnabled,
			);
			assert_eq!(
				EdfisSwapModule::get_target_amounts(&[EDF, USSD], 10000),
				Ok(vec![10000, 24874])
			);
			assert_eq!(
				EdfisSwapModule::get_target_amounts(&[EDF, USSD, WBTC], 10000),
				Ok(vec![10000, 24874, 1])
			);
			assert_noop!(
				EdfisSwapModule::get_target_amounts(&[EDF, USSD, WBTC], 100),
				Error::<Runtime>::ZeroTargetAmount,
			);
			assert_noop!(
				EdfisSwapModule::get_target_amounts(&[EDF, WBTC], 100),
				Error::<Runtime>::InsufficientLiquidity,
			);
		});
}

#[test]
fn calculate_amount_for_big_number_work() {
	ExtBuilder::default().build().execute_with(|| {
		LiquidityPool::<Runtime>::insert(
			USSDEDFPair::get(),
			(171_000_000_000_000_000_000_000, 56_000_000_000_000_000_000_000),
		);
		assert_eq!(
			EdfisSwapModule::get_supply_amount(
				171_000_000_000_000_000_000_000,
				56_000_000_000_000_000_000_000,
				1_000_000_000_000_000_000_000
			),
			3_140_495_867_768_595_041_323
		);
		assert_eq!(
			EdfisSwapModule::get_target_amount(
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
			LiquidityPool::<Runtime>::insert(USSDEDFPair::get(), (50000, 10000));
			LiquidityPool::<Runtime>::insert(USSDWBTCPair::get(), (100000, 10));
			assert_noop!(
				EdfisSwapModule::get_supply_amounts(&[EDF], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				EdfisSwapModule::get_supply_amounts(&[EDF, USSD, WBTC, EDF], 10000),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				EdfisSwapModule::get_supply_amounts(&[EDF, EDF], 10000),
				Error::<Runtime>::InvalidTradingPath,
			);
			assert_noop!(
				EdfisSwapModule::get_supply_amounts(&[EDF, USSD, EDF], 10000),
				Error::<Runtime>::InvalidTradingPath,
			);
			assert_noop!(
				EdfisSwapModule::get_supply_amounts(&[EDF, USSD, SEE], 10000),
				Error::<Runtime>::MustBeEnabled,
			);
			assert_eq!(
				EdfisSwapModule::get_supply_amounts(&[EDF, USSD], 24874),
				Ok(vec![10000, 24874])
			);
			assert_eq!(
				EdfisSwapModule::get_supply_amounts(&[EDF, USSD], 25000),
				Ok(vec![10102, 25000])
			);
			assert_noop!(
				EdfisSwapModule::get_supply_amounts(&[EDF, USSD, WBTC], 10000),
				Error::<Runtime>::ZeroSupplyAmount,
			);
			assert_noop!(
				EdfisSwapModule::get_supply_amounts(&[EDF, WBTC], 10000),
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
			LiquidityPool::<Runtime>::insert(USSDEDFPair::get(), (50000, 10000));

			assert_eq!(EdfisSwapModule::get_liquidity(USSD, EDF), (50000, 10000));
			assert_noop!(
				EdfisSwapModule::_swap(USSD, EDF, 50000, 5001),
				Error::<Runtime>::InvariantCheckFailed
			);
			assert_ok!(EdfisSwapModule::_swap(USSD, EDF, 50000, 5000));
			assert_eq!(EdfisSwapModule::get_liquidity(USSD, EDF), (100000, 5000));
			assert_ok!(EdfisSwapModule::_swap(EDF, USSD, 100, 800));
			assert_eq!(EdfisSwapModule::get_liquidity(USSD, EDF), (99200, 5100));
		});
}

#[test]
fn _swap_by_path_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(USSDEDFPair::get(), (50000, 10000));
			LiquidityPool::<Runtime>::insert(USSDWBTCPair::get(), (100000, 10));

			assert_eq!(EdfisSwapModule::get_liquidity(USSD, EDF), (50000, 10000));
			assert_eq!(EdfisSwapModule::get_liquidity(USSD, WBTC), (100000, 10));
			assert_ok!(EdfisSwapModule::_swap_by_path(&[EDF, USSD], &[10000, 25000]));
			assert_eq!(EdfisSwapModule::get_liquidity(USSD, EDF), (25000, 20000));
			assert_ok!(EdfisSwapModule::_swap_by_path(&[EDF, USSD, WBTC], &[100000, 20000, 1]));
			assert_eq!(EdfisSwapModule::get_liquidity(USSD, EDF), (5000, 120000));
			assert_eq!(EdfisSwapModule::get_liquidity(USSD, WBTC), (120000, 9));
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
				EdfisSwapModule::add_liquidity(
					RuntimeOrigin::signed(ALICE),
					SEE,
					USSD,
					100_000_000,
					100_000_000,
					0,
					false
				),
				Error::<Runtime>::MustBeEnabled
			);
			assert_noop!(
				EdfisSwapModule::add_liquidity(RuntimeOrigin::signed(ALICE), USSD, EDF, 0, 100_000_000, 0, false),
				Error::<Runtime>::InvalidLiquidityIncrement
			);

			assert_eq!(EdfisSwapModule::get_liquidity(USSD, EDF), (0, 0));
			assert_eq!(Tokens::free_balance(USSD, &EdfisSwapModule::account_id()), 0);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 0);
			assert_eq!(
				Tokens::free_balance(USSDEDFPair::get().dex_share_currency_id(), &ALICE),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(USSDEDFPair::get().dex_share_currency_id(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(USSD, &ALICE), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &ALICE), 1_000_000_000_000_000_000);

			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
				false,
			));
			System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::AddLiquidity {
				who: ALICE,
				currency_0: USSD,
				pool_0: 5_000_000_000_000,
				currency_1: EDF,
				pool_1: 1_000_000_000_000,
				share_increment: 10_000_000_000_000,
			}));
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, EDF),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(USSD, &EdfisSwapModule::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(USSDEDFPair::get().dex_share_currency_id(), &ALICE),
				10_000_000_000_000
			);
			assert_eq!(
				Tokens::reserved_balance(USSDEDFPair::get().dex_share_currency_id(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(USSD, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &ALICE), 999_999_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(USSDEDFPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(USSDEDFPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_eq!(Tokens::free_balance(USSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				EdfisSwapModule::add_liquidity(RuntimeOrigin::signed(BOB), USSD, EDF, 4, 1, 0, true,),
				Error::<Runtime>::InvalidLiquidityIncrement,
			);

			assert_noop!(
				EdfisSwapModule::add_liquidity(
					RuntimeOrigin::signed(BOB),
					USSD,
					EDF,
					50_000_000_000_000,
					8_000_000_000_000,
					80_000_000_000_001,
					true,
				),
				Error::<Runtime>::UnacceptableShareIncrement
			);

			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(BOB),
				USSD,
				EDF,
				50_000_000_000_000,
				8_000_000_000_000,
				80_000_000_000_000,
				true,
			));
			System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::AddLiquidity {
				who: BOB,
				currency_0: USSD,
				pool_0: 40_000_000_000_000,
				currency_1: EDF,
				pool_1: 8_000_000_000_000,
				share_increment: 80_000_000_000_000,
			}));
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, EDF),
				(45_000_000_000_000, 9_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(USSD, &EdfisSwapModule::account_id()), 45_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 9_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(USSDEDFPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(USSDEDFPair::get().dex_share_currency_id(), &BOB),
				80_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(USSD, &BOB), 999_960_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &BOB), 999_992_000_000_000_000);
		});
}

#[test]
fn remove_liquidity_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
				false
			));
			assert_noop!(
				EdfisSwapModule::remove_liquidity(
					RuntimeOrigin::signed(ALICE),
					USSDEDFPair::get().dex_share_currency_id(),
					EDF,
					100_000_000,
					0,
					0,
					false,
				),
				Error::<Runtime>::InvalidCurrencyId
			);

			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, EDF),
				(5_000_000_000_000, 1_000_000_000_000)
			);
			assert_eq!(Tokens::free_balance(USSD, &EdfisSwapModule::account_id()), 5_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 1_000_000_000_000);
			assert_eq!(
				Tokens::free_balance(USSDEDFPair::get().dex_share_currency_id(), &ALICE),
				10_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(USSD, &ALICE), 999_995_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &ALICE), 999_999_000_000_000_000);

			assert_noop!(
				EdfisSwapModule::remove_liquidity(
					RuntimeOrigin::signed(ALICE),
					USSD,
					EDF,
					8_000_000_000_000,
					4_000_000_000_001,
					800_000_000_000,
					false,
				),
				Error::<Runtime>::UnacceptableLiquidityWithdrawn
			);
			assert_noop!(
				EdfisSwapModule::remove_liquidity(
					RuntimeOrigin::signed(ALICE),
					USSD,
					EDF,
					8_000_000_000_000,
					4_000_000_000_000,
					800_000_000_001,
					false,
				),
				Error::<Runtime>::UnacceptableLiquidityWithdrawn
			);
			assert_ok!(EdfisSwapModule::remove_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				8_000_000_000_000,
				4_000_000_000_000,
				800_000_000_000,
				false,
			));
			System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::RemoveLiquidity {
				who: ALICE,
				currency_0: USSD,
				pool_0: 4_000_000_000_000,
				currency_1: EDF,
				pool_1: 800_000_000_000,
				share_decrement: 8_000_000_000_000,
			}));
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, EDF),
				(1_000_000_000_000, 200_000_000_000)
			);
			assert_eq!(Tokens::free_balance(USSD, &EdfisSwapModule::account_id()), 1_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 200_000_000_000);
			assert_eq!(
				Tokens::free_balance(USSDEDFPair::get().dex_share_currency_id(), &ALICE),
				2_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(USSD, &ALICE), 999_999_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &ALICE), 999_999_800_000_000_000);

			assert_ok!(EdfisSwapModule::remove_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				2_000_000_000_000,
				0,
				0,
				false,
			));
			System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::RemoveLiquidity {
				who: ALICE,
				currency_0: USSD,
				pool_0: 1_000_000_000_000,
				currency_1: EDF,
				pool_1: 200_000_000_000,
				share_decrement: 2_000_000_000_000,
			}));
			assert_eq!(EdfisSwapModule::get_liquidity(USSD, EDF), (0, 0));
			assert_eq!(Tokens::free_balance(USSD, &EdfisSwapModule::account_id()), 0);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 0);
			assert_eq!(
				Tokens::free_balance(USSDEDFPair::get().dex_share_currency_id(), &ALICE),
				0
			);
			assert_eq!(Tokens::free_balance(USSD, &ALICE), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &ALICE), 1_000_000_000_000_000_000);

			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(BOB),
				USSD,
				EDF,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
				true
			));
			assert_eq!(
				Tokens::free_balance(USSDEDFPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(USSDEDFPair::get().dex_share_currency_id(), &BOB),
				10_000_000_000_000
			);
			assert_ok!(EdfisSwapModule::remove_liquidity(
				RuntimeOrigin::signed(BOB),
				USSD,
				EDF,
				2_000_000_000_000,
				0,
				0,
				true,
			));
			assert_eq!(
				Tokens::free_balance(USSDEDFPair::get().dex_share_currency_id(), &BOB),
				0
			);
			assert_eq!(
				Tokens::reserved_balance(USSDEDFPair::get().dex_share_currency_id(), &BOB),
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

			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				500_000_000_000_000,
				100_000_000_000_000,
				0,
				false,
			));
			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				WBTC,
				100_000_000_000_000,
				10_000_000_000,
				0,
				false,
			));

			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, EDF),
				(500_000_000_000_000, 100_000_000_000_000)
			);
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, WBTC),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(USSD, &EdfisSwapModule::account_id()),
				600_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 100_000_000_000_000);
			assert_eq!(Tokens::free_balance(WBTC, &EdfisSwapModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(USSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(WBTC, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				EdfisSwapModule::do_swap_with_exact_supply(&BOB, &[EDF, USSD], 100_000_000_000_000, 250_000_000_000_000,),
				Error::<Runtime>::InsufficientTargetAmount
			);
			assert_noop!(
				EdfisSwapModule::do_swap_with_exact_supply(&BOB, &[EDF, USSD, WBTC, EDF], 100_000_000_000_000, 0),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				EdfisSwapModule::do_swap_with_exact_supply(&BOB, &[EDF, USSD, EDF], 100_000_000_000_000, 0),
				Error::<Runtime>::InvalidTradingPath,
			);
			assert_noop!(
				EdfisSwapModule::do_swap_with_exact_supply(&BOB, &[EDF, SEE], 100_000_000_000_000, 0),
				Error::<Runtime>::MustBeEnabled,
			);

			assert_ok!(EdfisSwapModule::do_swap_with_exact_supply(
				&BOB,
				&[EDF, USSD],
				100_000_000_000_000,
				200_000_000_000_000,
			));
			System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::Swap {
				trader: BOB,
				path: vec![EDF, USSD],
				liquidity_changes: vec![100_000_000_000_000, 248_743_718_592_964],
			}));
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, EDF),
				(251_256_281_407_036, 200_000_000_000_000)
			);
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, WBTC),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(USSD, &EdfisSwapModule::account_id()),
				351_256_281_407_036
			);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 200_000_000_000_000);
			assert_eq!(Tokens::free_balance(WBTC, &EdfisSwapModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(USSD, &BOB), 1_000_248_743_718_592_964);
			assert_eq!(Tokens::free_balance(EDF, &BOB), 999_900_000_000_000_000);
			assert_eq!(Tokens::free_balance(WBTC, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(EdfisSwapModule::do_swap_with_exact_supply(
				&BOB,
				&[EDF, USSD, WBTC],
				200_000_000_000_000,
				1,
			));
			System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::Swap {
				trader: BOB,
				path: vec![EDF, USSD, WBTC],
				liquidity_changes: vec![200_000_000_000_000, 124_996_843_514_053, 5_530_663_837],
			}));
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, EDF),
				(126_259_437_892_983, 400_000_000_000_000)
			);
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, WBTC),
				(224_996_843_514_053, 4_469_336_163)
			);
			assert_eq!(
				Tokens::free_balance(USSD, &EdfisSwapModule::account_id()),
				351_256_281_407_036
			);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 400_000_000_000_000);
			assert_eq!(Tokens::free_balance(WBTC, &EdfisSwapModule::account_id()), 4_469_336_163);
			assert_eq!(Tokens::free_balance(USSD, &BOB), 1_000_248_743_718_592_964);
			assert_eq!(Tokens::free_balance(EDF, &BOB), 999_700_000_000_000_000);
			assert_eq!(Tokens::free_balance(WBTC, &BOB), 1_000_000_005_530_663_837);
		});
}

#[test]
fn do_swap_with_exact_target_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				500_000_000_000_000,
				100_000_000_000_000,
				0,
				false,
			));
			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				WBTC,
				100_000_000_000_000,
				10_000_000_000,
				0,
				false,
			));

			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, EDF),
				(500_000_000_000_000, 100_000_000_000_000)
			);
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, WBTC),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(USSD, &EdfisSwapModule::account_id()),
				600_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 100_000_000_000_000);
			assert_eq!(Tokens::free_balance(WBTC, &EdfisSwapModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(USSD, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &BOB), 1_000_000_000_000_000_000);
			assert_eq!(Tokens::free_balance(WBTC, &BOB), 1_000_000_000_000_000_000);

			assert_noop!(
				EdfisSwapModule::do_swap_with_exact_target(&BOB, &[EDF, USSD], 250_000_000_000_000, 100_000_000_000_000,),
				Error::<Runtime>::ExcessiveSupplyAmount
			);
			assert_noop!(
				EdfisSwapModule::do_swap_with_exact_target(
					&BOB,
					&[EDF, USSD, WBTC, EDF],
					250_000_000_000_000,
					200_000_000_000_000,
				),
				Error::<Runtime>::InvalidTradingPathLength,
			);
			assert_noop!(
				EdfisSwapModule::do_swap_with_exact_target(&BOB, &[EDF, USSD, EDF], 250_000_000_000_000, 200_000_000_000_000,),
				Error::<Runtime>::InvalidTradingPath,
			);
			assert_noop!(
				EdfisSwapModule::do_swap_with_exact_target(&BOB, &[EDF, SEE], 250_000_000_000_000, 200_000_000_000_000),
				Error::<Runtime>::MustBeEnabled,
			);

			assert_ok!(EdfisSwapModule::do_swap_with_exact_target(
				&BOB,
				&[EDF, USSD],
				250_000_000_000_000,
				200_000_000_000_000,
			));
			System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::Swap {
				trader: BOB,
				path: vec![EDF, USSD],
				liquidity_changes: vec![101_010_101_010_102, 250_000_000_000_000],
			}));
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, EDF),
				(250_000_000_000_000, 201_010_101_010_102)
			);
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, WBTC),
				(100_000_000_000_000, 10_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(USSD, &EdfisSwapModule::account_id()),
				350_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 201_010_101_010_102);
			assert_eq!(Tokens::free_balance(WBTC, &EdfisSwapModule::account_id()), 10_000_000_000);
			assert_eq!(Tokens::free_balance(USSD, &BOB), 1_000_250_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &BOB), 999_898_989_898_989_898);
			assert_eq!(Tokens::free_balance(WBTC, &BOB), 1_000_000_000_000_000_000);

			assert_ok!(EdfisSwapModule::do_swap_with_exact_target(
				&BOB,
				&[EDF, USSD, WBTC],
				5_000_000_000,
				2_000_000_000_000_000,
			));
			System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::Swap {
				trader: BOB,
				path: vec![EDF, USSD, WBTC],
				liquidity_changes: vec![137_654_580_386_993, 101_010_101_010_102, 5_000_000_000],
			}));
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, EDF),
				(148_989_898_989_898, 338_664_681_397_095)
			);
			assert_eq!(
				EdfisSwapModule::get_liquidity(USSD, WBTC),
				(201_010_101_010_102, 5_000_000_000)
			);
			assert_eq!(
				Tokens::free_balance(USSD, &EdfisSwapModule::account_id()),
				350_000_000_000_000
			);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 338_664_681_397_095);
			assert_eq!(Tokens::free_balance(WBTC, &EdfisSwapModule::account_id()), 5_000_000_000);
			assert_eq!(Tokens::free_balance(USSD, &BOB), 1_000_250_000_000_000_000);
			assert_eq!(Tokens::free_balance(EDF, &BOB), 999_761_335_318_602_905);
			assert_eq!(Tokens::free_balance(WBTC, &BOB), 1_000_000_005_000_000_000);
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

			assert_eq!(EdfisSwapModule::get_liquidity(USSD, EDF), (1000000, 2000000));
			assert_eq!(Tokens::free_balance(USSD, &EdfisSwapModule::account_id()), 2000000);
			assert_eq!(Tokens::free_balance(EDF, &EdfisSwapModule::account_id()), 3000000);
			assert_eq!(
				Tokens::free_balance(USSDEDFPair::get().dex_share_currency_id(), &ALICE),
				2000000
			);
		});
}

#[test]
fn get_swap_amount_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(USSDEDFPair::get(), (50000, 10000));
			assert_eq!(
				EdfisSwapModule::get_swap_amount(&[EDF, USSD], SwapLimit::ExactSupply(10000, 0)),
				Some((10000, 24874))
			);
			assert_eq!(
				EdfisSwapModule::get_swap_amount(&[EDF, USSD], SwapLimit::ExactSupply(10000, 24875)),
				None
			);
			assert_eq!(
				EdfisSwapModule::get_swap_amount(&[EDF, USSD], SwapLimit::ExactTarget(Balance::max_value(), 24874)),
				Some((10000, 24874))
			);
			assert_eq!(
				EdfisSwapModule::get_swap_amount(&[EDF, USSD], SwapLimit::ExactTarget(9999, 24874)),
				None
			);
		});
}

#[test]
fn get_best_price_swap_path_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			LiquidityPool::<Runtime>::insert(USSDEDFPair::get(), (300000, 100000));
			LiquidityPool::<Runtime>::insert(USSDWBTCPair::get(), (50000, 10000));
			LiquidityPool::<Runtime>::insert(EDFWBTCPair::get(), (10000, 10000));

			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactSupply(10, 0), vec![]),
				Some((vec![EDF, USSD], 10, 29))
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactSupply(10, 30), vec![]),
				None
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactSupply(0, 0), vec![]),
				None
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactSupply(10, 0), vec![vec![SEE]]),
				Some((vec![EDF, USSD], 10, 29))
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactSupply(10, 0), vec![vec![EDF]]),
				Some((vec![EDF, USSD], 10, 29))
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactSupply(10, 0), vec![vec![USSD]]),
				Some((vec![EDF, USSD], 10, 29))
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactSupply(10, 0), vec![vec![WBTC]]),
				Some((vec![EDF, WBTC, USSD], 10, 44))
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactSupply(10000, 0), vec![vec![WBTC]]),
				Some((vec![EDF, USSD], 10000, 27024))
			);

			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactTarget(20, 30), vec![]),
				Some((vec![EDF, USSD], 11, 30))
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactTarget(10, 30), vec![]),
				None
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactTarget(0, 0), vec![]),
				None
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactTarget(20, 30), vec![vec![SEE]]),
				Some((vec![EDF, USSD], 11, 30))
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactTarget(20, 30), vec![vec![EDF]]),
				Some((vec![EDF, USSD], 11, 30))
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactTarget(20, 30), vec![vec![USSD]]),
				Some((vec![EDF, USSD], 11, 30))
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactTarget(20, 30), vec![vec![WBTC]]),
				Some((vec![EDF, WBTC, USSD], 8, 30))
			);
			assert_eq!(
				EdfisSwapModule::get_best_price_swap_path(EDF, USSD, SwapLimit::ExactTarget(100000, 20000), vec![vec![WBTC]]),
				Some((vec![EDF, USSD], 7216, 20000))
			);
		});
}

#[test]
fn swap_with_specific_path_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			System::set_block_number(1);
			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				500_000_000_000_000,
				100_000_000_000_000,
				0,
				false,
			));

			assert_noop!(
				EdfisSwapModule::swap_with_specific_path(
					&BOB,
					&[EDF, USSD],
					SwapLimit::ExactSupply(100_000_000_000_000, 248_743_718_592_965)
				),
				Error::<Runtime>::InsufficientTargetAmount
			);

			assert_ok!(EdfisSwapModule::swap_with_specific_path(
				&BOB,
				&[EDF, USSD],
				SwapLimit::ExactSupply(100_000_000_000_000, 200_000_000_000_000)
			));
			System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::Swap {
				trader: BOB,
				path: vec![EDF, USSD],
				liquidity_changes: vec![100_000_000_000_000, 248_743_718_592_964],
			}));

			assert_noop!(
				EdfisSwapModule::swap_with_specific_path(
					&BOB,
					&[USSD, EDF],
					SwapLimit::ExactTarget(253_794_223_643_470, 100_000_000_000_000)
				),
				Error::<Runtime>::ExcessiveSupplyAmount
			);

			assert_ok!(EdfisSwapModule::swap_with_specific_path(
				&BOB,
				&[USSD, EDF],
				SwapLimit::ExactTarget(300_000_000_000_000, 100_000_000_000_000)
			));
			System::assert_last_event(RuntimeEvent::EdfisSwapModule(crate::Event::Swap {
				trader: BOB,
				path: vec![USSD, EDF],
				liquidity_changes: vec![253_794_223_643_471, 100_000_000_000_000],
			}));
		});
}

#[test]
fn get_liquidity_token_address_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Disabled
		);
		assert_eq!(EdfisSwapModule::get_liquidity_token_address(USSD, EDF), None);

		assert_ok!(EdfisSwapModule::list_provisioning(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF,
			1_000_000_000_000u128,
			1_000_000_000_000u128,
			5_000_000_000_000u128,
			2_000_000_000_000u128,
			10,
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
				accumulated_provision: (0, 0),
				not_before: 10,
			})
		);
		assert_eq!(
			EdfisSwapModule::get_liquidity_token_address(USSD, EDF),
			Some(H160::from_str("0x0000000000000000000200000000010000000002").unwrap())
		);

		assert_ok!(EdfisSwapModule::enable_trading_pair(
			RuntimeOrigin::signed(ListingOrigin::get()),
			USSD,
			EDF
		));
		assert_eq!(
			EdfisSwapModule::trading_pair_statuses(USSDEDFPair::get()),
			TradingPairStatus::<_, _>::Enabled
		);
		assert_eq!(
			EdfisSwapModule::get_liquidity_token_address(USSD, EDF),
			Some(H160::from_str("0x0000000000000000000200000000010000000002").unwrap())
		);
	});
}

#[test]
fn specific_joint_swap_work() {
	ExtBuilder::default()
		.initialize_enabled_trading_pairs()
		.build()
		.execute_with(|| {
			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				EDF,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
				false,
			));
			assert_ok!(EdfisSwapModule::add_liquidity(
				RuntimeOrigin::signed(ALICE),
				USSD,
				WBTC,
				5_000_000_000_000,
				1_000_000_000_000,
				0,
				false,
			));

			assert_eq!(
				USSDJointSwap::get_swap_amount(WBTC, EDF, SwapLimit::ExactSupply(10000, 0)),
				Some((10000, 9800))
			);
			assert_eq!(
				SEEJointSwap::get_swap_amount(WBTC, EDF, SwapLimit::ExactSupply(10000, 0)),
				None
			);

			assert_noop!(
				USSDJointSwap::swap(&CAROL, WBTC, EDF, SwapLimit::ExactSupply(10000, 0)),
				orml_tokens::Error::<Runtime>::BalanceTooLow,
			);
			assert_noop!(
				USSDJointSwap::swap(&BOB, WBTC, EDF, SwapLimit::ExactSupply(10000, 9801)),
				SwapError::CannotSwap,
			);
			assert_noop!(
				SEEJointSwap::swap(&BOB, WBTC, EDF, SwapLimit::ExactSupply(10000, 0)),
				SwapError::CannotSwap,
			);

			assert_eq!(
				USSDJointSwap::swap(&BOB, WBTC, EDF, SwapLimit::ExactSupply(10000, 0)),
				Ok((10000, 9800)),
			);

			assert_eq!(
				USSDJointSwap::swap(&BOB, EDF, WBTC, SwapLimit::ExactTarget(20000, 10000)),
				Ok((10204, 10000)),
			);
		});
}
