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

//! Unit tests for the SERP Treasury module.

#![cfg(test)]

use super::*;
use frame_support::assert_ok;
use mock::*;

// #[test]
// fn set_stable_currency_inflation_rate_works() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_eq!(SerpTreasuryModule::stable_currency_inflation_rate(SETUSD), 0);
// 		assert_ok!(SerpTreasuryModule::set_stable_currency_inflation_rate(Origin::signed(ALICE), SerpStableCurrencyId::SETUSD, 1));
// 		assert_eq!(SerpTreasuryModule::stable_currency_inflation_rate(SETUSD), 1);
// 	});
// }

// #[test]
// fn force_serpdown_works() {

// }

// #[test]
// fn serp_tes_now() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_eq!(SerpTreasuryModule::serp_tes_now(), 0);
// 		assert_ok!(SerpTreasuryModule::serp_tes_now(1));
// 		assert_eq!(SerpTreasuryModule::serp_tes_now(), 1);
// 	});
// }

// #[test]
// fn issue_stablecurrency_inflation_works() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_eq!(Currencies::total_issuance(SETUSD), );
// 		assert_ok!(SerpTreasuryModule::issue_stablecurrency_inflation());
// 		// get SETUSD total_issuance
// 		assert_eq!(Currencies::total_issuance(SETUSD), );
// 		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 2000);
// 	});
// }

// #[test]
// fn get_buyback_serpup_works() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_eq!(SerpTreasuryModule::get_buyback_serpup(SETUSD, &ALICE), 0);
// 		assert_ok!(SerpTreasuryModule::set_buyback_serpup(SETUSD, &ALICE, 1000));
// 		assert_eq!(SerpTreasuryModule::get_buyback_serpup(SETUSD, &ALICE), 1000);
// 	});
// }

// #[test]
// fn add_cashdrop_to_pool_works() {

// }

// #[test]
// fn issue_cashdrop_from_pool_works() {
	
// }

// #[test]
// fn get_cashdrop_serpup_works() {
	
// }

// #[test]
// fn get_buyback_serplus_works() {
	
// }

// #[test]
// fn get_cashdrop_serplus_works() {
	
// }

// #[test]
// fn on_serplus_works() {
	
// }

// #[test]
// fn on_serpup_works() {
	
// }

// #[test]
// fn on_serpdown_works() {
	
// }

// #[test]
// fn get_minimum_supply_works() {
	
// }

#[test]
fn issue_standard_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 1000);

		assert_ok!(SerpTreasuryModule::issue_standard(SETUSD, &ALICE, 1000));
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 2000);

		assert_ok!(SerpTreasuryModule::issue_standard(SETUSD, &ALICE, 1000));
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 3000);
	});
}

#[test]
fn burn_standard_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 1000);
		assert_ok!(SerpTreasuryModule::burn_standard(SETUSD, &ALICE, 300));
		assert_eq!(Currencies::free_balance(SETUSD, &ALICE), 700);
	});
}

#[test]
fn issue_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 1000);

		assert_ok!(SerpTreasuryModule::issue_setter(&ALICE, 1000));
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 2000);

		assert_ok!(SerpTreasuryModule::issue_setter(&ALICE, 1000));
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 3000);
	});
}

#[test]
fn burn_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 1000);
		assert_ok!(SerpTreasuryModule::burn_setter(&ALICE, 300));
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 700);
	});
}

#[test]
fn deposit_setter_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Currencies::free_balance(SETR, &SerpTreasuryModule::account_id()), 0);
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 1000);
		assert_eq!(SerpTreasuryModule::deposit_setter(&ALICE, 10000).is_ok(), false);
		assert_ok!(SerpTreasuryModule::deposit_setter(&ALICE, 500));
		assert_eq!(Currencies::free_balance(SETR, &SerpTreasuryModule::account_id()), 500);
		assert_eq!(Currencies::free_balance(SETR, &ALICE), 500);
	});
}

// #[test]
// fn claim_cashdrop_works() {
	
// }

// #[test]
// fn buyback_swap_with_exact_supply_works() {
	
// }

// #[test]
// fn buyback_swap_with_exact_target_works() {
	
// }
