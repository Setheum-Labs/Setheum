// // This file is part of Setheum.

// // Copyright (C) 2020-2022 Setheum Labs.
// // SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// // This program is free software: you can redistribute it and/or modify
// // it under the terms of the GNU General Public License as published by
// // the Free Software Foundation, either version 3 of the License, or
// // (at your option) any later version.

// // This program is distributed in the hope that it will be useful,
// // but WITHOUT ANY WARRANTY; without even the implied warranty of
// // MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// // GNU General Public License for more details.

// // You should have received a copy of the GNU General Public License
// // along with this program. If not, see <https://www.gnu.org/licenses/>.

// //! Unit tests for the dex module.

// #![cfg(test)]

// use super::*;
// use frame_support::{assert_noop, assert_ok};
// use mock::{
// 	USDIETHPair, USDIWBTCPair, WBTCETHPair, DexModule, Event, ExtBuilder, ListingOrigin, Origin, Runtime, System, Tokens,
// 	SETM, ALICE, USDI, BOB, ETH, WBTC,
// };
// use orml_traits::MultiReservableCurrency;
// use sp_core::H160;
// use sp_runtime::traits::BadOrigin;
// use std::str::FromStr;

// #[test]
// fn list_provisioning_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);

// 		assert_noop!(
// 			DexModule::list_provisioning(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				1_000_000_000_000u128,
// 				1_000_000_000_000u128,
// 				5_000_000_000_000u128,
// 				2_000_000_000_000u128,
// 				10,
// 			),
// 			BadOrigin
// 		);

// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Disabled
// 		);
// 		assert_ok!(DexModule::list_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC,
// 			1_000_000_000_000u128,
// 			1_000_000_000_000u128,
// 			5_000_000_000_000u128,
// 			2_000_000_000_000u128,
// 			10,
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
// 				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
// 				target_provision: (2_000_000_000_000u128, 5_000_000_000_000u128),
// 				accumulated_provision: (0, 0),
// 				not_before: 10,
// 			})
// 		);
// 		System::assert_last_event(Event::DexModule(crate::Event::ListProvisioning {
// 			trading_pair: USDIWBTCPair::get(),
// 		}));

// 		assert_noop!(
// 			DexModule::list_provisioning(
// 				Origin::signed(ListingOrigin::get()),
// 				USDI,
// 				USDI,
// 				1_000_000_000_000u128,
// 				1_000_000_000_000u128,
// 				5_000_000_000_000u128,
// 				2_000_000_000_000u128,
// 				10,
// 			),
// 			Error::<Runtime>::InvalidCurrencyId
// 		);

// 		assert_noop!(
// 			DexModule::list_provisioning(
// 				Origin::signed(ListingOrigin::get()),
// 				USDI,
// 				WBTC,
// 				1_000_000_000_000u128,
// 				1_000_000_000_000u128,
// 				5_000_000_000_000u128,
// 				2_000_000_000_000u128,
// 				10,
// 			),
// 			Error::<Runtime>::MustBeDisabled
// 		);
// 	});
// }

// #[test]
// fn update_provisioning_parameters_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);

// 		assert_noop!(
// 			DexModule::update_provisioning_parameters(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				1_000_000_000_000u128,
// 				1_000_000_000_000u128,
// 				5_000_000_000_000u128,
// 				2_000_000_000_000u128,
// 				10,
// 			),
// 			BadOrigin
// 		);

// 		assert_noop!(
// 			DexModule::update_provisioning_parameters(
// 				Origin::signed(ListingOrigin::get()),
// 				USDI,
// 				WBTC,
// 				1_000_000_000_000u128,
// 				1_000_000_000_000u128,
// 				5_000_000_000_000u128,
// 				2_000_000_000_000u128,
// 				10,
// 			),
// 			Error::<Runtime>::MustBeProvisioning
// 		);

// 		assert_ok!(DexModule::list_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC,
// 			1_000_000_000_000u128,
// 			1_000_000_000_000u128,
// 			5_000_000_000_000u128,
// 			2_000_000_000_000u128,
// 			10,
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
// 				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
// 				target_provision: (2_000_000_000_000u128, 5_000_000_000_000u128),
// 				accumulated_provision: (0, 0),
// 				not_before: 10,
// 			})
// 		);

// 		assert_ok!(DexModule::update_provisioning_parameters(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC,
// 			2_000_000_000_000u128,
// 			0,
// 			3_000_000_000_000u128,
// 			2_000_000_000_000u128,
// 			50,
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
// 				min_contribution: (2_000_000_000_000u128, 0),
// 				target_provision: (3_000_000_000_000u128, 2_000_000_000_000u128),
// 				accumulated_provision: (0, 0),
// 				not_before: 50,
// 			})
// 		);
// 	});
// }

// #[test]
// fn enable_diabled_trading_pair_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);

// 		assert_noop!(
// 			DexModule::enable_trading_pair(Origin::signed(ALICE), USDI, WBTC),
// 			BadOrigin
// 		);

// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Disabled
// 		);
// 		assert_ok!(DexModule::enable_trading_pair(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Enabled
// 		);
// 		System::assert_last_event(Event::DexModule(crate::Event::EnableTradingPair {
// 			trading_pair: USDIWBTCPair::get(),
// 		}));

// 		assert_noop!(
// 			DexModule::enable_trading_pair(Origin::signed(ListingOrigin::get()), WBTC, USDI),
// 			Error::<Runtime>::AlreadyEnabled
// 		);
// 	});
// }

// #[test]
// fn enable_provisioning_without_provision_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);

// 		assert_ok!(DexModule::list_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC,
// 			1_000_000_000_000u128,
// 			1_000_000_000_000u128,
// 			5_000_000_000_000u128,
// 			2_000_000_000_000u128,
// 			10,
// 		));
// 		assert_ok!(DexModule::list_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			ETH,
// 			1_000_000_000_000u128,
// 			1_000_000_000_000u128,
// 			5_000_000_000_000u128,
// 			2_000_000_000_000u128,
// 			10,
// 		));
// 		assert_ok!(DexModule::add_provision(
// 			Origin::signed(ALICE),
// 			USDI,
// 			ETH,
// 			1_000_000_000_000u128,
// 			1_000_000_000_000u128
// 		));

// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
// 				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
// 				target_provision: (2_000_000_000_000u128, 5_000_000_000_000u128),
// 				accumulated_provision: (0, 0),
// 				not_before: 10,
// 			})
// 		);
// 		assert_ok!(DexModule::enable_trading_pair(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Enabled
// 		);
// 		System::assert_last_event(Event::DexModule(crate::Event::EnableTradingPair {
// 			trading_pair: USDIWBTCPair::get(),
// 		}));

// 		assert_noop!(
// 			DexModule::enable_trading_pair(Origin::signed(ListingOrigin::get()), USDI, ETH),
// 			Error::<Runtime>::StillProvisioning
// 		);
// 	});
// }

// #[test]
// fn end_provisioning_trading_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);

// 		assert_ok!(DexModule::list_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC,
// 			1_000_000_000_000u128,
// 			1_000_000_000_000u128,
// 			5_000_000_000_000u128,
// 			2_000_000_000_000u128,
// 			10,
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
// 				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
// 				target_provision: (2_000_000_000_000u128, 5_000_000_000_000u128),
// 				accumulated_provision: (0, 0),
// 				not_before: 10,
// 			})
// 		);

// 		assert_ok!(DexModule::list_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			ETH,
// 			1_000_000_000_000u128,
// 			1_000_000_000_000u128,
// 			5_000_000_000_000u128,
// 			2_000_000_000_000u128,
// 			10,
// 		));
// 		assert_ok!(DexModule::add_provision(
// 			Origin::signed(ALICE),
// 			USDI,
// 			ETH,
// 			1_000_000_000_000u128,
// 			2_000_000_000_000u128
// 		));

// 		assert_noop!(
// 			DexModule::end_provisioning(Origin::signed(ListingOrigin::get()), USDI, ETH),
// 			Error::<Runtime>::UnqualifiedProvision
// 		);
// 		System::set_block_number(10);

// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIETHPair::get()),
// 			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
// 				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
// 				target_provision: (2_000_000_000_000u128, 5_000_000_000_000u128),
// 				accumulated_provision: (2_000_000_000_000u128, 1_000_000_000_000u128),
// 				not_before: 10,
// 			})
// 		);
// 		assert_eq!(
// 			DexModule::initial_share_exchange_rates(USDIETHPair::get()),
// 			Default::default()
// 		);
// 		assert_eq!(DexModule::liquidity_pool(USDIETHPair::get()), (0, 0));
// 		assert_eq!(Tokens::total_issuance(USDIETHPair::get().dex_share_currency_id()), 0);
// 		assert_eq!(
// 			Tokens::free_balance(USDIETHPair::get().dex_share_currency_id(), &DexModule::account_id()),
// 			0
// 		);

// 		assert_ok!(DexModule::end_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			ETH
// 		));
// 		System::assert_last_event(Event::DexModule(crate::Event::ProvisioningToEnabled {
// 			trading_pair: USDIETHPair::get(),
// 			pool_0: 1_000_000_000_000u128,
// 			pool_1: 2_000_000_000_000u128,
// 			share_amount: 2_000_000_000_000u128,
// 		}));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIETHPair::get()),
// 			TradingPairStatus::<_, _>::Enabled
// 		);
// 		assert_eq!(
// 			DexModule::initial_share_exchange_rates(USDIETHPair::get()),
// 			(ExchangeRate::one(), ExchangeRate::checked_from_rational(1, 2).unwrap())
// 		);
// 		assert_eq!(
// 			DexModule::liquidity_pool(USDIETHPair::get()),
// 			(1_000_000_000_000u128, 2_000_000_000_000u128)
// 		);
// 		assert_eq!(
// 			Tokens::total_issuance(USDIETHPair::get().dex_share_currency_id()),
// 			2_000_000_000_000u128
// 		);
// 		assert_eq!(
// 			Tokens::free_balance(USDIETHPair::get().dex_share_currency_id(), &DexModule::account_id()),
// 			2_000_000_000_000u128
// 		);
// 	});
// }

// #[test]
// fn disable_trading_pair_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);

// 		assert_ok!(DexModule::enable_trading_pair(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Enabled
// 		);

// 		assert_noop!(
// 			DexModule::disable_trading_pair(Origin::signed(ALICE), USDI, WBTC),
// 			BadOrigin
// 		);

// 		assert_ok!(DexModule::disable_trading_pair(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Disabled
// 		);
// 		System::assert_last_event(Event::DexModule(crate::Event::DisableTradingPair {
// 			trading_pair: USDIWBTCPair::get(),
// 		}));

// 		assert_noop!(
// 			DexModule::disable_trading_pair(Origin::signed(ListingOrigin::get()), USDI, WBTC),
// 			Error::<Runtime>::MustBeEnabled
// 		);

// 		assert_ok!(DexModule::list_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			ETH,
// 			1_000_000_000_000u128,
// 			1_000_000_000_000u128,
// 			5_000_000_000_000u128,
// 			2_000_000_000_000u128,
// 			10,
// 		));
// 		assert_noop!(
// 			DexModule::disable_trading_pair(Origin::signed(ListingOrigin::get()), USDI, ETH),
// 			Error::<Runtime>::MustBeEnabled
// 		);
// 	});
// }

// #[test]
// fn add_provision_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);

// 		assert_noop!(
// 			DexModule::add_provision(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				5_000_000_000_000u128,
// 				1_000_000_000_000u128,
// 			),
// 			Error::<Runtime>::MustBeProvisioning
// 		);

// 		assert_ok!(DexModule::list_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC,
// 			5_000_000_000_000u128,
// 			1_000_000_000_000u128,
// 			5_000_000_000_000_000u128,
// 			1_000_000_000_000_000u128,
// 			10,
// 		));

// 		assert_noop!(
// 			DexModule::add_provision(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				4_999_999_999_999u128,
// 				999_999_999_999u128,
// 			),
// 			Error::<Runtime>::InvalidContributionIncrement
// 		);

// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
// 				min_contribution: (1_000_000_000_000u128, 5_000_000_000_000u128),
// 				target_provision: (1_000_000_000_000_000u128, 5_000_000_000_000_000u128),
// 				accumulated_provision: (0, 0),
// 				not_before: 10,
// 			})
// 		);
// 		assert_eq!(DexModule::provisioning_pool(USDIWBTCPair::get(), ALICE), (0, 0));
// 		assert_eq!(Tokens::free_balance(USDI, &ALICE), 1_000_000_000_000_000_000u128);
// 		assert_eq!(Tokens::free_balance(WBTC, &ALICE), 1_000_000_000_000_000_000u128);
// 		assert_eq!(Tokens::free_balance(USDI, &DexModule::account_id()), 0);
// 		assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 0);
// 		let alice_ref_count_0 = System::consumers(&ALICE);

// 		assert_ok!(DexModule::add_provision(
// 			Origin::signed(ALICE),
// 			USDI,
// 			WBTC,
// 			5_000_000_000_000u128,
// 			0,
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
// 				min_contribution: (1_000_000_000_000u128, 5_000_000_000_000u128),
// 				target_provision: (1_000_000_000_000_000u128, 5_000_000_000_000_000u128),
// 				accumulated_provision: (0, 5_000_000_000_000u128),
// 				not_before: 10,
// 			})
// 		);
// 		assert_eq!(
// 			DexModule::provisioning_pool(USDIWBTCPair::get(), ALICE),
// 			(0, 5_000_000_000_000u128)
// 		);
// 		assert_eq!(Tokens::free_balance(USDI, &ALICE), 999_995_000_000_000_000u128);
// 		assert_eq!(Tokens::free_balance(WBTC, &ALICE), 1_000_000_000_000_000_000u128);
// 		assert_eq!(
// 			Tokens::free_balance(USDI, &DexModule::account_id()),
// 			5_000_000_000_000u128
// 		);
// 		assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 0);
// 		let alice_ref_count_1 = System::consumers(&ALICE);
// 		assert_eq!(alice_ref_count_1, alice_ref_count_0 + 1);
// 		System::assert_last_event(Event::DexModule(crate::Event::AddProvision {
// 			who: ALICE,
// 			currency_0: USDI,
// 			contribution_0: 5_000_000_000_000u128,
// 			currency_1: WBTC,
// 			contribution_1: 0,
// 		}));
// 	});
// }

// #[test]
// fn claim_dex_share_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);

// 		assert_ok!(DexModule::list_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC,
// 			5_000_000_000_000u128,
// 			1_000_000_000_000u128,
// 			5_000_000_000_000_000u128,
// 			1_000_000_000_000_000u128,
// 			0,
// 		));

// 		assert_ok!(DexModule::add_provision(
// 			Origin::signed(ALICE),
// 			USDI,
// 			WBTC,
// 			1_000_000_000_000_000u128,
// 			200_000_000_000_000u128,
// 		));
// 		assert_ok!(DexModule::add_provision(
// 			Origin::signed(BOB),
// 			USDI,
// 			WBTC,
// 			4_000_000_000_000_000u128,
// 			800_000_000_000_000u128,
// 		));

// 		assert_noop!(
// 			DexModule::claim_dex_share(Origin::signed(ALICE), ALICE, USDI, WBTC),
// 			Error::<Runtime>::StillProvisioning
// 		);

// 		assert_ok!(DexModule::end_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC
// 		));

// 		let lp_currency_id = USDIWBTCPair::get().dex_share_currency_id();

// 		assert!(InitialShareExchangeRates::<Runtime>::contains_key(USDIWBTCPair::get()),);
// 		assert_eq!(
// 			DexModule::initial_share_exchange_rates(USDIWBTCPair::get()),
// 			(ExchangeRate::one(), ExchangeRate::saturating_from_rational(2, 10))
// 		);
// 		assert_eq!(
// 			Tokens::free_balance(lp_currency_id, &DexModule::account_id()),
// 			2_000_000_000_000_000u128
// 		);
// 		assert_eq!(
// 			DexModule::provisioning_pool(USDIWBTCPair::get(), ALICE),
// 			(1_000_000_000_000_000u128, 200_000_000_000_000u128)
// 		);
// 		assert_eq!(
// 			DexModule::provisioning_pool(USDIWBTCPair::get(), BOB),
// 			(4_000_000_000_000_000u128, 800_000_000_000_000u128)
// 		);
// 		assert_eq!(Tokens::free_balance(lp_currency_id, &ALICE), 0);
// 		assert_eq!(Tokens::free_balance(lp_currency_id, &BOB), 0);

// 		let alice_ref_count_0 = System::consumers(&ALICE);
// 		let bob_ref_count_0 = System::consumers(&BOB);

// 		assert_ok!(DexModule::claim_dex_share(Origin::signed(ALICE), ALICE, USDI, WBTC));
// 		assert_eq!(
// 			Tokens::free_balance(lp_currency_id, &DexModule::account_id()),
// 			8_000_000_000_000_000u128
// 		);
// 		assert_eq!(DexModule::provisioning_pool(USDIWBTCPair::get(), ALICE), (0, 0));
// 		assert_eq!(Tokens::free_balance(lp_currency_id, &ALICE), 2_000_000_000_000_000u128);
// 		assert_eq!(System::consumers(&ALICE), alice_ref_count_0 - 1);
// 		assert!(InitialShareExchangeRates::<Runtime>::contains_key(USDIWBTCPair::get()),);

// 		assert_ok!(DexModule::disable_trading_pair(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC
// 		));
// 		assert_ok!(DexModule::claim_dex_share(Origin::signed(BOB), BOB, USDI, WBTC));
// 		assert_eq!(Tokens::free_balance(lp_currency_id, &DexModule::account_id()), 0);
// 		assert_eq!(DexModule::provisioning_pool(USDIWBTCPair::get(), BOB), (0, 0));
// 		assert_eq!(Tokens::free_balance(lp_currency_id, &BOB), 8_000_000_000_000_000u128);
// 		assert_eq!(System::consumers(&BOB), bob_ref_count_0 - 1);
// 		assert!(!InitialShareExchangeRates::<Runtime>::contains_key(USDIWBTCPair::get()),);
// 	});
// }

// #[test]
// fn get_liquidity_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		LiquidityPool::<Runtime>::insert(USDIWBTCPair::get(), (1000, 20));
// 		assert_eq!(DexModule::liquidity_pool(USDIWBTCPair::get()), (1000, 20));
// 		assert_eq!(DexModule::get_liquidity(USDI, WBTC), (20, 1000));
// 		assert_eq!(DexModule::get_liquidity(WBTC, USDI), (1000, 20));
// 	});
// }

// #[test]
// fn get_target_amount_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_eq!(DexModule::get_target_amount(10000, 0, 1000), 0);
// 		assert_eq!(DexModule::get_target_amount(0, 20000, 1000), 0);
// 		assert_eq!(DexModule::get_target_amount(10000, 20000, 0), 0);
// 		assert_eq!(DexModule::get_target_amount(10000, 1, 1000000), 0);
// 		assert_eq!(DexModule::get_target_amount(10000, 20000, 10000), 9974);
// 		assert_eq!(DexModule::get_target_amount(10000, 20000, 1000), 1809);
// 	});
// }

// #[test]
// fn get_supply_amount_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_eq!(DexModule::get_supply_amount(10000, 0, 1000), 0);
// 		assert_eq!(DexModule::get_supply_amount(0, 20000, 1000), 0);
// 		assert_eq!(DexModule::get_supply_amount(10000, 20000, 0), 0);
// 		assert_eq!(DexModule::get_supply_amount(10000, 1, 1), 0);
// 		assert_eq!(DexModule::get_supply_amount(10000, 20000, 9949), 9949);
// 		assert_eq!(DexModule::get_target_amount(10000, 20000, 9999), 9974);
// 		assert_eq!(DexModule::get_supply_amount(10000, 20000, 1801), 995);
// 		assert_eq!(DexModule::get_target_amount(10000, 20000, 1000), 1801);
// 	});
// }

// #[test]
// fn get_target_amounts_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			LiquidityPool::<Runtime>::insert(USDIWBTCPair::get(), (50000, 10000));
// 			LiquidityPool::<Runtime>::insert(USDIETHPair::get(), (100000, 10));
// 			assert_noop!(
// 				DexModule::get_target_amounts(&[WBTC], 10000),
// 				Error::<Runtime>::InvalidTradingPathLength,
// 			);
// 			assert_noop!(
// 				DexModule::get_target_amounts(&[WBTC, USDI, ETH, WBTC], 10000),
// 				Error::<Runtime>::InvalidTradingPath,
// 			);
// 			assert_noop!(
// 				DexModule::get_target_amounts(&[WBTC, WBTC], 10000),
// 				Error::<Runtime>::InvalidTradingPath,
// 			);
// 			assert_noop!(
// 				DexModule::get_target_amounts(&[WBTC, USDI, WBTC], 10000),
// 				Error::<Runtime>::InvalidTradingPath,
// 			);
// 			assert_noop!(
// 				DexModule::get_target_amounts(&[WBTC, USDI, SETM], 10000),
// 				Error::<Runtime>::MustBeEnabled,
// 			);
// 			assert_eq!(
// 				DexModule::get_target_amounts(&[WBTC, USDI], 10000),
// 				Ok(vec![10000, 1659])
// 			);
// 			assert_eq!(
// 				DexModule::get_target_amounts(&[WBTC, USDI, ETH], 10000),
// 				Ok(vec![10000, 1659, 99397])
// 			);
// 			assert_noop!(
// 				DexModule::get_target_amounts(&[WBTC, USDI, ETH], 100),
// 				Error::<Runtime>::ZeroTargetAmount,
// 			);
// 			assert_noop!(
// 				DexModule::get_target_amounts(&[WBTC, ETH], 100),
// 				Error::<Runtime>::InsufficientLiquidity,
// 			);
// 		});
// }

// #[test]
// fn calculate_amount_for_big_number_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		LiquidityPool::<Runtime>::insert(
// 			USDIWBTCPair::get(),
// 			(171_000_000_000_000_000_000_000, 56_000_000_000_000_000_000_000),
// 		);
// 		assert_eq!(
// 			DexModule::get_supply_amount(
// 				171_000_000_000_000_000_000_000,
// 				56_000_000_000_000_000_000_000,
// 				1_000_000_000_000_000_000_000
// 			),
// 			3124714481498401096392
// 		);
// 		assert_eq!(
// 			DexModule::get_target_amount(
// 				171_000_000_000_000_000_000_000,
// 				56_000_000_000_000_000_000_000,
// 				3124714481498401096392
// 			),
// 			1_000_000_000_000_000_000_000
// 		);
// 	});
// }

// #[test]
// fn get_supply_amounts_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			LiquidityPool::<Runtime>::insert(USDIWBTCPair::get(), (50000, 10000));
// 			LiquidityPool::<Runtime>::insert(USDIETHPair::get(), (100000, 10));
// 			assert_noop!(
// 				DexModule::get_supply_amounts(&[WBTC], 10000),
// 				Error::<Runtime>::InvalidTradingPathLength,
// 			);
// 			assert_noop!(
// 				DexModule::get_supply_amounts(&[WBTC, WBTC], 10000),
// 				Error::<Runtime>::InvalidTradingPath,
// 			);
// 			assert_noop!(
// 				DexModule::get_supply_amounts(&[WBTC, USDI, WBTC], 10000),
// 				Error::<Runtime>::InvalidTradingPath,
// 			);
// 			assert_noop!(
// 				DexModule::get_supply_amounts(&[WBTC, USDI, SETM], 10000),
// 				Error::<Runtime>::MustBeEnabled,
// 			);
// 			assert_eq!(
// 				DexModule::get_supply_amounts(&[WBTC, USDI], 24874),
// 				Ok(vec![10000, 24874])
// 			);
// 			assert_eq!(
// 				DexModule::get_supply_amounts(&[WBTC, USDI], 25000),
// 				Ok(vec![10102, 25000])
// 			);
// 			assert_noop!(
// 				DexModule::get_supply_amounts(&[WBTC, USDI, ETH], 10000),
// 				Error::<Runtime>::ZeroSupplyAmount,
// 			);
// 			assert_noop!(
// 				DexModule::get_supply_amounts(&[WBTC, ETH], 10000),
// 				Error::<Runtime>::InsufficientLiquidity,
// 			);
// 		});
// }

// #[test]
// fn _swap_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			LiquidityPool::<Runtime>::insert(USDIWBTCPair::get(), (50000, 10000));

// 			assert_eq!(DexModule::get_liquidity(USDI, WBTC), (10000, 50000));
// 			assert_noop!(
// 				DexModule::_swap(USDI, WBTC, 50000, 5001),
// 				Error::<Runtime>::InvariantCheckFailed
// 			);
// 			assert_ok!(DexModule::_swap(USDI, WBTC, 50000, 5000));
// 			assert_eq!(DexModule::get_liquidity(USDI, WBTC), (100000, 5000));
// 			assert_ok!(DexModule::_swap(WBTC, USDI, 100, 800));
// 			assert_eq!(DexModule::get_liquidity(USDI, WBTC), (99200, 5100));
// 		});
// }

// #[test]
// fn _swap_by_path_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			LiquidityPool::<Runtime>::insert(USDIWBTCPair::get(), (50000, 10000));
// 			LiquidityPool::<Runtime>::insert(USDIETHPair::get(), (100000, 10));

// 			assert_eq!(DexModule::get_liquidity(USDI, WBTC), (10000, 50000));
// 			assert_eq!(DexModule::get_liquidity(USDI, ETH), (10, 100000));
// 			assert_ok!(DexModule::_swap_by_path(&[WBTC, USDI], &[10000, 25000]));
// 			assert_eq!(DexModule::get_liquidity(USDI, WBTC), (25000, 20000));
// 			assert_ok!(DexModule::_swap_by_path(&[WBTC, USDI, ETH], &[100000, 20000, 1]));
// 			assert_eq!(DexModule::get_liquidity(USDI, WBTC), (5000, 120000));
// 			assert_eq!(DexModule::get_liquidity(USDI, ETH), (120000, 9));
// 		});
// }

// #[test]
// fn add_liquidity_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			System::set_block_number(1);

// 			assert_noop!(
// 				DexModule::add_liquidity(Origin::signed(ALICE), SETM, USDI, 100_000_000, 100_000_000, 0),
// 				Error::<Runtime>::MustBeEnabled
// 			);
// 			assert_noop!(
// 				DexModule::add_liquidity(Origin::signed(ALICE), USDI, WBTC, 0, 100_000_000, 0),
// 				Error::<Runtime>::InvalidLiquidityIncrement
// 			);

// 			assert_eq!(DexModule::get_liquidity(USDI, WBTC), (0, 0));
// 			assert_eq!(Tokens::free_balance(USDI, &DexModule::account_id()), 0);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 0);
// 			assert_eq!(
// 				Tokens::free_balance(USDIWBTCPair::get().dex_share_currency_id(), &ALICE),
// 				0
// 			);
// 			assert_eq!(
// 				Tokens::reserved_balance(USDIWBTCPair::get().dex_share_currency_id(), &ALICE),
// 				0
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &ALICE), 1_000_000_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &ALICE), 1_000_000_000_000_000_000);

// 			assert_ok!(DexModule::add_liquidity(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				5_000_000_000_000,
// 				1_000_000_000_000,
// 				0,
// 			));
// 			System::assert_last_event(Event::DexModule(crate::Event::AddLiquidity {
// 				who: ALICE,
// 				currency_0: USDI,
// 				pool_0: 5_000_000_000_000,
// 				currency_1: WBTC,
// 				pool_1: 1_000_000_000_000,
// 				share_increment: 10_000_000_000_000,
// 			}));
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, WBTC),
// 				(5_000_000_000_000, 1_000_000_000_000)
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &DexModule::account_id()), 5_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 1_000_000_000_000);
// 			assert_eq!(
// 				Tokens::free_balance(USDIWBTCPair::get().dex_share_currency_id(), &ALICE),
// 				10_000_000_000_000
// 			);
// 			assert_eq!(
// 				Tokens::reserved_balance(USDIWBTCPair::get().dex_share_currency_id(), &ALICE),
// 				0
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &ALICE), 999_995_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &ALICE), 999_999_000_000_000_000);
// 			assert_eq!(
// 				Tokens::free_balance(USDIWBTCPair::get().dex_share_currency_id(), &BOB),
// 				0
// 			);
// 			assert_eq!(
// 				Tokens::reserved_balance(USDIWBTCPair::get().dex_share_currency_id(), &BOB),
// 				0
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &BOB), 1_000_000_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &BOB), 1_000_000_000_000_000_000);

// 			assert_noop!(
// 				DexModule::add_liquidity(Origin::signed(BOB), USDI, WBTC, 4, 1, 0),
// 				Error::<Runtime>::InvalidLiquidityIncrement,
// 			);

// 			assert_noop!(
// 				DexModule::add_liquidity(
// 					Origin::signed(BOB),
// 					USDI,
// 					WBTC,
// 					50_000_000_000_000,
// 					8_000_000_000_000,
// 					80_000_000_000_001,
// 				),
// 				Error::<Runtime>::UnacceptableShareIncrement
// 			);

// 			assert_ok!(DexModule::add_liquidity(
// 				Origin::signed(BOB),
// 				USDI,
// 				WBTC,
// 				50_000_000_000_000,
// 				8_000_000_000_000,
// 				80_000_000_000_000,
// 			));
// 			System::assert_last_event(Event::DexModule(crate::Event::AddLiquidity {
// 				who: BOB,
// 				currency_0: USDI,
// 				pool_0: 40_000_000_000_000,
// 				currency_1: WBTC,
// 				pool_1: 8_000_000_000_000,
// 				share_increment: 80_000_000_000_000,
// 			}));
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, WBTC),
// 				(45_000_000_000_000, 9_000_000_000_000)
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &DexModule::account_id()), 45_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 9_000_000_000_000);
// 			assert_eq!(
// 				Tokens::free_balance(USDIWBTCPair::get().dex_share_currency_id(), &BOB),
// 				0
// 			);
// 			assert_eq!(
// 				Tokens::reserved_balance(USDIWBTCPair::get().dex_share_currency_id(), &BOB),
// 				80_000_000_000_000
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &BOB), 999_960_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &BOB), 999_992_000_000_000_000);
// 		});
// }

// #[test]
// fn remove_liquidity_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			System::set_block_number(1);

// 			assert_ok!(DexModule::add_liquidity(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				5_000_000_000_000,
// 				1_000_000_000_000,
// 				0,
// 			));
// 			assert_noop!(
// 				DexModule::remove_liquidity(
// 					Origin::signed(ALICE),
// 					USDIWBTCPair::get().dex_share_currency_id(),
// 					WBTC,
// 					100_000_000,
// 					0,
// 					0,
// 				),
// 				Error::<Runtime>::InvalidCurrencyId
// 			);

// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, WBTC),
// 				(5_000_000_000_000, 1_000_000_000_000)
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &DexModule::account_id()), 5_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 1_000_000_000_000);
// 			assert_eq!(
// 				Tokens::free_balance(USDIWBTCPair::get().dex_share_currency_id(), &ALICE),
// 				2000000000000
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &ALICE), 999_995_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &ALICE), 999_999_000_000_000_000);

// 			// assert_noop!(
// 			// 	DexModule::remove_liquidity(
// 			// 		Origin::signed(ALICE),
// 			// 		USDI,
// 			// 		WBTC,
// 			// 		8_000_000_000_000,
// 			// 		4_000_000_000_001,
// 			// 		800_000_000_000,
// 			// 	),
// 			// 	Error::<Runtime>::UnacceptableLiquidityWithdrawn
// 			// );
// 			// assert_noop!(
// 			// 	DexModule::remove_liquidity(
// 			// 		Origin::signed(ALICE),
// 			// 		USDI,
// 			// 		WBTC,
// 			// 		8_000_000_000_000,
// 			// 		4_000_000_000_000,
// 			// 		800_000_000_001,
// 			// 	),
// 			// 	Error::<Runtime>::UnacceptableLiquidityWithdrawn
// 			// );
// 			assert_ok!(DexModule::remove_liquidity(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				8_000_000_000_000,
// 				4_000_000_000_000,
// 				800_000_000_000,
// 			));
// 			System::assert_last_event(Event::DexModule(crate::Event::RemoveLiquidity {
// 				who: ALICE,
// 				currency_0: USDI,
// 				pool_0: 4_000_000_000_000,
// 				currency_1: WBTC,
// 				pool_1: 800_000_000_000,
// 				share_decrement: 8_000_000_000_000,
// 			}));
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, WBTC),
// 				(1_000_000_000_000, 200_000_000_000)
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &DexModule::account_id()), 1_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 200_000_000_000);
// 			assert_eq!(
// 				Tokens::free_balance(USDIWBTCPair::get().dex_share_currency_id(), &ALICE),
// 				2_000_000_000_000
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &ALICE), 999_999_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &ALICE), 999_999_800_000_000_000);

// 			assert_ok!(DexModule::remove_liquidity(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				2_000_000_000_000,
// 				0,
// 				0,
// 			));
// 			System::assert_last_event(Event::DexModule(crate::Event::RemoveLiquidity {
// 				who: ALICE,
// 				currency_0: USDI,
// 				pool_0: 1_000_000_000_000,
// 				currency_1: WBTC,
// 				pool_1: 200_000_000_000,
// 				share_decrement: 2_000_000_000_000,
// 			}));
// 			assert_eq!(DexModule::get_liquidity(USDI, WBTC), (0, 0));
// 			assert_eq!(Tokens::free_balance(USDI, &DexModule::account_id()), 0);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 0);
// 			assert_eq!(
// 				Tokens::free_balance(USDIWBTCPair::get().dex_share_currency_id(), &ALICE),
// 				0
// 			);
// 			assert_eq!(Tokens::free_balance(USDI, &ALICE), 1_000_000_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &ALICE), 1_000_000_000_000_000_000);

// 			assert_ok!(DexModule::add_liquidity(
// 				Origin::signed(BOB),
// 				USDI,
// 				WBTC,
// 				5_000_000_000_000,
// 				1_000_000_000_000,
// 				0,
// 			));
// 			assert_eq!(
// 				Tokens::free_balance(USDIWBTCPair::get().dex_share_currency_id(), &BOB),
// 				0
// 			);
// 			assert_eq!(
// 				Tokens::reserved_balance(USDIWBTCPair::get().dex_share_currency_id(), &BOB),
// 				10_000_000_000_000
// 			);
// 			assert_ok!(DexModule::remove_liquidity(
// 				Origin::signed(BOB),
// 				USDI,
// 				WBTC,
// 				2_000_000_000_000,
// 				0,
// 				0,
// 			));
// 			assert_eq!(
// 				Tokens::free_balance(USDIWBTCPair::get().dex_share_currency_id(), &BOB),
// 				0
// 			);
// 			assert_eq!(
// 				Tokens::reserved_balance(USDIWBTCPair::get().dex_share_currency_id(), &BOB),
// 				8_000_000_000_000
// 			);
// 		});
// }

// #[test]
// fn do_swap_with_exact_supply_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			System::set_block_number(1);

// 			assert_ok!(DexModule::add_liquidity(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				500_000_000_000_000,
// 				100_000_000_000_000,
// 				0,
// 			));
// 			assert_ok!(DexModule::add_liquidity(
// 				Origin::signed(ALICE),
// 				USDI,
// 				ETH,
// 				100_000_000_000_000,
// 				10_000_000_000,
// 				0,
// 			));

// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, WBTC),
// 				(500_000_000_000_000, 100_000_000_000_000)
// 			);
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, ETH),
// 				(100_000_000_000_000, 10_000_000_000)
// 			);
// 			assert_eq!(
// 				Tokens::free_balance(USDI, &DexModule::account_id()),
// 				600_000_000_000_000
// 			);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 100_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(ETH, &DexModule::account_id()), 10_000_000_000);
// 			assert_eq!(Tokens::free_balance(USDI, &BOB), 1_000_000_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &BOB), 1_000_000_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(ETH, &BOB), 1_000_000_000_000_000_000);

// 			assert_noop!(
// 				DexModule::do_swap_with_exact_supply(&BOB, &[WBTC, USDI], 100_000_000_000_000, 250_000_000_000_000,),
// 				Error::<Runtime>::InsufficientTargetAmount
// 			);
// 			// assert_noop!(
// 			// 	DexModule::do_swap_with_exact_supply(&BOB, &[WBTC, USDI, ETH, WBTC], 100_000_000_000_000, 0),
// 			// 	Error::<Runtime>::InvalidTradingPathLength,
// 			// );
// 			assert_noop!(
// 				DexModule::do_swap_with_exact_supply(&BOB, &[WBTC, USDI, WBTC], 100_000_000_000_000, 0),
// 				Error::<Runtime>::InvalidTradingPath,
// 			);
// 			assert_noop!(
// 				DexModule::do_swap_with_exact_supply(&BOB, &[WBTC, SETM], 100_000_000_000_000, 0),
// 				Error::<Runtime>::MustBeEnabled,
// 			);

// 			assert_ok!(DexModule::do_swap_with_exact_supply(
// 				&BOB,
// 				&[WBTC, USDI],
// 				100_000_000_000_000,
// 				200_000_000_000_000,
// 			));
// 			System::assert_last_event(Event::DexModule(crate::Event::Swap {
// 				trader: BOB,
// 				path: vec![WBTC, USDI],
// 				liquidity_changes: vec![100_000_000_000_000, 248_743_718_592_964],
// 			}));
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, WBTC),
// 				(251_256_281_407_036, 200_000_000_000_000)
// 			);
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, ETH),
// 				(100_000_000_000_000, 10_000_000_000)
// 			);
// 			assert_eq!(
// 				Tokens::free_balance(USDI, &DexModule::account_id()),
// 				351_256_281_407_036
// 			);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 200_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(ETH, &DexModule::account_id()), 10_000_000_000);
// 			assert_eq!(Tokens::free_balance(USDI, &BOB), 1_000_248_743_718_592_964);
// 			assert_eq!(Tokens::free_balance(WBTC, &BOB), 999_900_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(ETH, &BOB), 1_000_000_000_000_000_000);

// 			assert_ok!(DexModule::do_swap_with_exact_supply(
// 				&BOB,
// 				&[WBTC, USDI, ETH],
// 				200_000_000_000_000,
// 				1,
// 			));
// 			System::assert_last_event(Event::DexModule(crate::Event::Swap {
// 				trader: BOB,
// 				path: vec![WBTC, USDI, ETH],
// 				liquidity_changes: vec![200_000_000_000_000, 124_996_843_514_053, 5_530_663_837],
// 			}));
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, WBTC),
// 				(126_259_437_892_983, 400_000_000_000_000)
// 			);
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, ETH),
// 				(224_996_843_514_053, 4_469_336_163)
// 			);
// 			assert_eq!(
// 				Tokens::free_balance(USDI, &DexModule::account_id()),
// 				351_256_281_407_036
// 			);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 400_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(ETH, &DexModule::account_id()), 4_469_336_163);
// 			assert_eq!(Tokens::free_balance(USDI, &BOB), 1_000_248_743_718_592_964);
// 			assert_eq!(Tokens::free_balance(WBTC, &BOB), 999_700_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(ETH, &BOB), 1_000_000_005_530_663_837);
// 		});
// }

// #[test]
// fn do_swap_with_exact_target_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			System::set_block_number(1);

// 			assert_ok!(DexModule::add_liquidity(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				500_000_000_000_000,
// 				100_000_000_000_000,
// 				0,
// 			));
// 			assert_ok!(DexModule::add_liquidity(
// 				Origin::signed(ALICE),
// 				USDI,
// 				ETH,
// 				100_000_000_000_000,
// 				10_000_000_000,
// 				0,
// 			));

// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, WBTC),
// 				(500_000_000_000_000, 100_000_000_000_000)
// 			);
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, ETH),
// 				(100_000_000_000_000, 10_000_000_000)
// 			);
// 			assert_eq!(
// 				Tokens::free_balance(USDI, &DexModule::account_id()),
// 				600_000_000_000_000
// 			);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 100_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(ETH, &DexModule::account_id()), 10_000_000_000);
// 			assert_eq!(Tokens::free_balance(USDI, &BOB), 1_000_000_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &BOB), 1_000_000_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(ETH, &BOB), 1_000_000_000_000_000_000);

// 			assert_noop!(
// 				DexModule::do_swap_with_exact_target(&BOB, &[WBTC, USDI], 250_000_000_000_000, 100_000_000_000_000,),
// 				Error::<Runtime>::ExcessiveSupplyAmount
// 			);
// 			assert_noop!(
// 				DexModule::do_swap_with_exact_target(
// 					&BOB,
// 					&[WBTC, USDI, ETH, WBTC],
// 					250_000_000_000_000,
// 					200_000_000_000_000,
// 				),
// 				Error::<Runtime>::InvalidTradingPathLength,
// 			);
// 			assert_noop!(
// 				DexModule::do_swap_with_exact_target(&BOB, &[WBTC, USDI, WBTC], 250_000_000_000_000, 200_000_000_000_000,),
// 				Error::<Runtime>::InvalidTradingPath,
// 			);
// 			assert_noop!(
// 				DexModule::do_swap_with_exact_target(&BOB, &[WBTC, SETM], 250_000_000_000_000, 200_000_000_000_000),
// 				Error::<Runtime>::MustBeEnabled,
// 			);

// 			assert_ok!(DexModule::do_swap_with_exact_target(
// 				&BOB,
// 				&[WBTC, USDI],
// 				250_000_000_000_000,
// 				200_000_000_000_000,
// 			));
// 			System::assert_last_event(Event::DexModule(crate::Event::Swap {
// 				trader: BOB,
// 				path: vec![WBTC, USDI],
// 				liquidity_changes: vec![101_010_101_010_102, 250_000_000_000_000],
// 			}));
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, WBTC),
// 				(250_000_000_000_000, 201_010_101_010_102)
// 			);
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, ETH),
// 				(100_000_000_000_000, 10_000_000_000)
// 			);
// 			assert_eq!(
// 				Tokens::free_balance(USDI, &DexModule::account_id()),
// 				350_000_000_000_000
// 			);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 201_010_101_010_102);
// 			assert_eq!(Tokens::free_balance(ETH, &DexModule::account_id()), 10_000_000_000);
// 			assert_eq!(Tokens::free_balance(USDI, &BOB), 1_000_250_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &BOB), 999_898_989_898_989_898);
// 			assert_eq!(Tokens::free_balance(ETH, &BOB), 1_000_000_000_000_000_000);

// 			assert_ok!(DexModule::do_swap_with_exact_target(
// 				&BOB,
// 				&[WBTC, USDI, ETH],
// 				5_000_000_000,
// 				2_000_000_000_000_000,
// 			));
// 			System::assert_last_event(Event::DexModule(crate::Event::Swap {
// 				trader: BOB,
// 				path: vec![WBTC, USDI, ETH],
// 				liquidity_changes: vec![137_654_580_386_993, 101_010_101_010_102, 5_000_000_000],
// 			}));
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, WBTC),
// 				(148_989_898_989_898, 338_664_681_397_095)
// 			);
// 			assert_eq!(
// 				DexModule::get_liquidity(USDI, ETH),
// 				(201_010_101_010_102, 5_000_000_000)
// 			);
// 			assert_eq!(
// 				Tokens::free_balance(USDI, &DexModule::account_id()),
// 				350_000_000_000_000
// 			);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 338_664_681_397_095);
// 			assert_eq!(Tokens::free_balance(ETH, &DexModule::account_id()), 5_000_000_000);
// 			assert_eq!(Tokens::free_balance(USDI, &BOB), 1_000_250_000_000_000_000);
// 			assert_eq!(Tokens::free_balance(WBTC, &BOB), 999_761_335_318_602_905);
// 			assert_eq!(Tokens::free_balance(ETH, &BOB), 1_000_000_005_000_000_000);
// 		});
// }

// #[test]
// fn initialize_added_liquidity_pools_genesis_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.initialize_added_liquidity_pools(ALICE)
// 		.build()
// 		.execute_with(|| {
// 			System::set_block_number(1);

// 			assert_eq!(DexModule::get_liquidity(USDI, WBTC), (2000000, 1000000));
// 			assert_eq!(Tokens::free_balance(USDI, &DexModule::account_id()), 4000000);
// 			assert_eq!(Tokens::free_balance(WBTC, &DexModule::account_id()), 3000000);
// 			assert_eq!(
// 				Tokens::free_balance(USDIWBTCPair::get().dex_share_currency_id(), &ALICE),
// 				2000000
// 			);
// 		});
// }

// #[test]
// fn get_swap_amount_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			LiquidityPool::<Runtime>::insert(USDIWBTCPair::get(), (50000, 10000));
// 			assert_eq!(
// 				DexModule::get_swap_amount(&vec![WBTC, USDI], SwapLimit::ExactSupply(10000, 0)),
// 				Some((10000, 24874))
// 			);
// 			assert_eq!(
// 				DexModule::get_swap_amount(&vec![WBTC, USDI], SwapLimit::ExactSupply(10000, 1659)),
// 				None
// 			);
// 			assert_eq!(
// 				DexModule::get_swap_amount(&vec![WBTC, USDI], SwapLimit::ExactTarget(Balance::max_value(), 24874)),
// 				Some((10000, 24874))
// 			);
// 			assert_eq!(
// 				DexModule::get_swap_amount(&vec![WBTC, USDI], SwapLimit::ExactTarget(9999, 24874)),
// 				None
// 			);
// 		});
// }

// #[test]
// fn get_best_price_swap_path_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			LiquidityPool::<Runtime>::insert(USDIWBTCPair::get(), (300000, 100000));
// 			LiquidityPool::<Runtime>::insert(USDIETHPair::get(), (50000, 10000));
// 			LiquidityPool::<Runtime>::insert(WBTCETHPair::get(), (10000, 10000));

// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactSupply(10, 0), vec![]),
// 				Some(vec![WBTC, USDI])
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactSupply(10, 30), vec![]),
// 				None
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactSupply(0, 0), vec![]),
// 				None
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactSupply(10, 0), vec![vec![SETM]]),
// 				Some(vec![WBTC, USDI])
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactSupply(10, 0), vec![vec![WBTC]]),
// 				Some(vec![WBTC, USDI])
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactSupply(10, 0), vec![vec![USDI]]),
// 				Some(vec![WBTC, USDI])
// 			);
// 			// assert_eq!(
// 			// 	DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactSupply(10, 0), vec![vec![ETH]]),
// 			// 	Some(vec![WBTC, ETH, USDI])
// 			// );
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactSupply(10000, 0), vec![vec![ETH]]),
// 				Some(vec![WBTC, USDI])
// 			);

// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactTarget(20, 30), vec![]),
// 				Some(vec![WBTC, USDI])
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactTarget(10, 30), vec![]),
// 				None
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactTarget(0, 0), vec![]),
// 				None
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactTarget(20, 30), vec![vec![SETM]]),
// 				Some(vec![WBTC, USDI])
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactTarget(20, 30), vec![vec![WBTC]]),
// 				Some(vec![WBTC, USDI])
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactTarget(20, 30), vec![vec![USDI]]),
// 				Some(vec![WBTC, USDI])
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactTarget(20, 30), vec![vec![ETH]]),
// 				Some(vec![WBTC, ETH, USDI])
// 			);
// 			assert_eq!(
// 				DexModule::get_best_price_swap_path(WBTC, USDI, SwapLimit::ExactTarget(100000, 20000), vec![vec![ETH]]),
// 				Some(vec![WBTC, USDI])
// 			);
// 		});
// }

// #[test]
// fn swap_with_specific_path_work() {
// 	ExtBuilder::default()
// 		.initialize_enabled_trading_pairs()
// 		.build()
// 		.execute_with(|| {
// 			System::set_block_number(1);
// 			assert_ok!(DexModule::add_liquidity(
// 				Origin::signed(ALICE),
// 				USDI,
// 				WBTC,
// 				500_000_000_000_000,
// 				100_000_000_000_000,
// 				0,
// 			));

// 			assert_ok!(DexModule::swap_with_specific_path(
// 					&BOB,
// 					&vec![WBTC, USDI],
// 					SwapLimit::ExactSupply(100_000_000_000_000, 249373433583959)
// 			));

// 			assert_ok!(DexModule::swap_with_specific_path(
// 				&BOB,
// 				&vec![WBTC, USDI],
// 				SwapLimit::ExactSupply(100_000_000_000_000, 200_000_000_000_000)
// 			));
// 			System::assert_last_event(Event::DexModule(crate::Event::Swap {
// 				trader: BOB,
// 				path: vec![WBTC, USDI],
// 				liquidity_changes: vec![100_000_000_000_000, 248_743_718_592_964],
// 			}));

// 			assert_noop!(
// 				DexModule::swap_with_specific_path(
// 					&BOB,
// 					&vec![USDI, WBTC],
// 					SwapLimit::ExactTarget(253_794_223_643_470, 100_000_000_000_000)
// 				),
// 				Error::<Runtime>::ExcessiveSupplyAmount
// 			);

// 			assert_ok!(DexModule::swap_with_specific_path(
// 				&BOB,
// 				&vec![USDI, WBTC],
// 				SwapLimit::ExactTarget(300_000_000_000_000, 100_000_000_000_000)
// 			));
// 			System::assert_last_event(Event::DexModule(crate::Event::Swap {
// 				trader: BOB,
// 				path: vec![USDI, WBTC],
// 				liquidity_changes: vec![253_794_223_643_471, 100_000_000_000_000],
// 			}));
// 		});
// }

// #[test]
// fn get_liquidity_token_address_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		System::set_block_number(1);

// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Disabled
// 		);
// 		assert_eq!(DexModule::get_liquidity_token_address(USDI, WBTC), None);

// 		assert_ok!(DexModule::list_provisioning(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC,
// 			1_000_000_000_000u128,
// 			1_000_000_000_000u128,
// 			5_000_000_000_000u128,
// 			2_000_000_000_000u128,
// 			10,
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Provisioning(ProvisioningParameters {
// 				min_contribution: (1_000_000_000_000u128, 1_000_000_000_000u128),
// 				target_provision: (5_000_000_000_000u128, 2_000_000_000_000u128),
// 				accumulated_provision: (0, 0),
// 				not_before: 10,
// 			})
// 		);
// 		assert_eq!(
// 			DexModule::get_liquidity_token_address(USDI, WBTC),
// 			Some(H160::from_str("0x0000000000000000000200000000010000000002").unwrap())
// 		);

// 		assert_ok!(DexModule::enable_trading_pair(
// 			Origin::signed(ListingOrigin::get()),
// 			USDI,
// 			WBTC
// 		));
// 		assert_eq!(
// 			DexModule::trading_pair_statuses(USDIWBTCPair::get()),
// 			TradingPairStatus::<_, _>::Enabled
// 		);
// 		assert_eq!(
// 			DexModule::get_liquidity_token_address(USDI, WBTC),
// 			Some(H160::from_str("0x0000000000000000000200000000010000000002").unwrap())
// 		);
// 	});
// }
