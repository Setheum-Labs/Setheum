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

//! Unit tests for the auction manager module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

#[test]
fn get_auction_time_to_close_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(AuctionManagerModule::get_auction_time_to_close(2000, 1), 100);
		assert_eq!(AuctionManagerModule::get_auction_time_to_close(2001, 1), 50);
	});
}

#[test]
fn setter_auction_methods() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AuctionManagerModule::new_setter_auction(&ALICE, BTC, 10, 100));
		let setter_auction_with_positive_target = AuctionManagerModule::setter_auctions(0).unwrap();
		assert_eq!(setter_auction_with_positive_target.always_forward(), false);
		assert_eq!(setter_auction_with_positive_target.in_reverse_stage(99), false);
		assert_eq!(setter_auction_with_positive_target.in_reverse_stage(100), true);
		assert_eq!(setter_auction_with_positive_target.in_reverse_stage(101), true);
		assert_eq!(setter_auction_with_positive_target.payment_amount(99), 99);
		assert_eq!(setter_auction_with_positive_target.payment_amount(100), 100);
		assert_eq!(setter_auction_with_positive_target.payment_amount(101), 100);
		assert_eq!(setter_auction_with_positive_target.reserve_amount(80, 100), 10);
		assert_eq!(setter_auction_with_positive_target.reserve_amount(100, 200), 5);

		assert_ok!(AuctionManagerModule::new_setter_auction(&ALICE, BTC, 10, 0));
		let setter_auction_with_zero_target = AuctionManagerModule::setter_auctions(1).unwrap();
		assert_eq!(setter_auction_with_zero_target.always_forward(), true);
		assert_eq!(setter_auction_with_zero_target.in_reverse_stage(0), false);
		assert_eq!(setter_auction_with_zero_target.in_reverse_stage(100), false);
		assert_eq!(setter_auction_with_zero_target.payment_amount(99), 99);
		assert_eq!(setter_auction_with_zero_target.payment_amount(101), 101);
		assert_eq!(setter_auction_with_zero_target.reserve_amount(100, 200), 10);
	});
}

#[test]
fn diamond_auction_methods() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AuctionManagerModule::new_diamond_auction(200, 100));
		let diamond_auction = AuctionManagerModule::diamond_auctions(0).unwrap();
		assert_eq!(diamond_auction.amount_for_sale(0, 100), 200);
		assert_eq!(diamond_auction.amount_for_sale(100, 200), 100);
		assert_eq!(diamond_auction.amount_for_sale(200, 1000), 40);
	});
}

#[test]
fn new_setter_auction_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let ref_count_0 = System::consumers(&ALICE);
		assert_noop!(
			AuctionManagerModule::new_setter_auction(&ALICE, BTC, 0, 100),
			Error::<Runtime>::InvalidAmount,
		);

		assert_ok!(AuctionManagerModule::new_setter_auction(&ALICE, BTC, 10, 100));
		let new_setter_auction_event = Event::auction_manager(crate::Event::NewSetterAuction(0, BTC, 10, 100));
		assert!(System::events()
			.iter()
			.any(|record| record.event == new_setter_auction_event));

		assert_eq!(AuctionManagerModule::total_reserve_in_auction(BTC), 10);
		assert_eq!(AuctionManagerModule::total_target_in_auction(), 100);
		assert_eq!(AuctionModule::auctions_index(), 1);
		assert_eq!(System::consumers(&ALICE), ref_count_0 + 1);

		assert_noop!(
			AuctionManagerModule::new_setter_auction(&ALICE, BTC, Balance::max_value(), Balance::max_value()),
			Error::<Runtime>::InvalidAmount,
		);
	});
}

#[test]
fn new_diamond_auction_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			AuctionManagerModule::new_diamond_auction(0, 100),
			Error::<Runtime>::InvalidAmount,
		);
		assert_noop!(
			AuctionManagerModule::new_diamond_auction(200, 0),
			Error::<Runtime>::InvalidAmount,
		);

		assert_ok!(AuctionManagerModule::new_diamond_auction(200, 100));
		let new_diamond_auction_event = Event::auction_manager(crate::Event::NewDiamondAuction(0, 200, 100));
		assert!(System::events()
			.iter()
			.any(|record| record.event == new_diamond_auction_event));

		assert_eq!(AuctionManagerModule::total_standard_in_auction(), 100);
		assert_eq!(AuctionModule::auctions_index(), 1);

		assert_noop!(
			AuctionManagerModule::new_diamond_auction(200, Balance::max_value()),
			Error::<Runtime>::InvalidAmount,
		);
	});
}

#[test]
fn new_serplus_auction_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			AuctionManagerModule::new_serplus_auction(0),
			Error::<Runtime>::InvalidAmount,
		);

		assert_ok!(AuctionManagerModule::new_serplus_auction(100));
		let new_serplus_auction_event = Event::auction_manager(crate::Event::NewSerplusAuction(0, 100));
		assert!(System::events()
			.iter()
			.any(|record| record.event == new_serplus_auction_event));

		assert_eq!(AuctionManagerModule::total_serplusin_auction(), 100);
		assert_eq!(AuctionModule::auctions_index(), 1);

		assert_noop!(
			AuctionManagerModule::new_serplus_auction(Balance::max_value()),
			Error::<Runtime>::InvalidAmount,
		);
	});
}

#[test]
fn setter_auction_bid_handler_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AuctionManagerModule::setter_auction_bid_handler(1, 0, (BOB, 4), None),
			Error::<Runtime>::AuctionNotExists,
		);

		assert_ok!(SerpTreasuryModule::deposit_reserve(&ALICE, BTC, 10));
		assert_ok!(AuctionManagerModule::new_setter_auction(&ALICE, BTC, 10, 100));
		assert_eq!(SerpTreasuryModule::serpluspool(), 0);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 1000);

		let bob_ref_count_0 = System::consumers(&BOB);

		assert_noop!(
			AuctionManagerModule::setter_auction_bid_handler(1, 0, (BOB, 4), None),
			Error::<Runtime>::InvalidBidPrice,
		);
		assert_eq!(
			AuctionManagerModule::setter_auction_bid_handler(1, 0, (BOB, 5), None).is_ok(),
			true
		);
		assert_eq!(SerpTreasuryModule::serpluspool(), 5);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 995);

		let bob_ref_count_1 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_1, bob_ref_count_0 + 1);
		let carol_ref_count_0 = System::consumers(&CAROL);

		assert_eq!(
			AuctionManagerModule::setter_auction_bid_handler(2, 0, (CAROL, 10), Some((BOB, 5))).is_ok(),
			true
		);
		assert_eq!(SerpTreasuryModule::serpluspool(), 10);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 1000);
		assert_eq!(Tokens::free_balance(USDJ, &CAROL), 990);
		assert_eq!(AuctionManagerModule::setter_auctions(0).unwrap().amount, 10);

		let bob_ref_count_2 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_2, bob_ref_count_1 - 1);
		let carol_ref_count_1 = System::consumers(&CAROL);
		assert_eq!(carol_ref_count_1, carol_ref_count_0 + 1);

		assert_eq!(
			AuctionManagerModule::setter_auction_bid_handler(3, 0, (BOB, 200), Some((CAROL, 10))).is_ok(),
			true
		);
		assert_eq!(SerpTreasuryModule::serpluspool(), 100);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 900);
		assert_eq!(Tokens::free_balance(USDJ, &CAROL), 1000);
		assert_eq!(AuctionManagerModule::setter_auctions(0).unwrap().amount, 5);

		let bob_ref_count_3 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_3, bob_ref_count_2 + 1);
		let carol_ref_count_2 = System::consumers(&CAROL);
		assert_eq!(carol_ref_count_2, carol_ref_count_1 - 1);
	});
}

#[test]
fn diamond_auction_bid_handler_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AuctionManagerModule::diamond_auction_bid_handler(1, 0, (BOB, 99), None),
			Error::<Runtime>::AuctionNotExists,
		);

		assert_ok!(AuctionManagerModule::new_diamond_auction(200, 100));
		assert_eq!(AuctionManagerModule::total_standard_in_auction(), 100);
		assert_eq!(AuctionManagerModule::diamond_auctions(0).unwrap().amount, 200);
		assert_eq!(SerpTreasuryModule::serpluspool(), 0);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 1000);

		let bob_ref_count_0 = System::consumers(&BOB);

		assert_noop!(
			AuctionManagerModule::diamond_auction_bid_handler(1, 0, (BOB, 99), None),
			Error::<Runtime>::InvalidBidPrice,
		);
		assert_eq!(
			AuctionManagerModule::diamond_auction_bid_handler(1, 0, (BOB, 100), None).is_ok(),
			true
		);
		assert_eq!(AuctionManagerModule::diamond_auctions(0).unwrap().amount, 200);
		assert_eq!(SerpTreasuryModule::serpluspool(), 100);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 900);

		let bob_ref_count_1 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_1, bob_ref_count_0 + 1);
		let carol_ref_count_0 = System::consumers(&CAROL);

		assert_eq!(
			AuctionManagerModule::diamond_auction_bid_handler(2, 0, (CAROL, 200), Some((BOB, 100))).is_ok(),
			true
		);
		assert_eq!(AuctionManagerModule::diamond_auctions(0).unwrap().amount, 100);
		assert_eq!(SerpTreasuryModule::serpluspool(), 100);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 1000);
		assert_eq!(Tokens::free_balance(USDJ, &CAROL), 900);
		let bob_ref_count_2 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_2, bob_ref_count_1 - 1);
		let carol_ref_count_1 = System::consumers(&CAROL);
		assert_eq!(carol_ref_count_1, carol_ref_count_0 + 1);
	});
}

#[test]
fn serplus_auction_bid_handler_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AuctionManagerModule::serplus_auction_bid_handler(1, 0, (BOB, 99), None),
			Error::<Runtime>::AuctionNotExists,
		);

		assert_ok!(AuctionManagerModule::new_serplus_auction(100));
		assert_eq!(Tokens::free_balance(DNAR, &BOB), 1000);

		let bob_ref_count_0 = System::consumers(&BOB);

		assert_eq!(
			AuctionManagerModule::serplus_auction_bid_handler(1, 0, (BOB, 50), None).is_ok(),
			true
		);
		assert_eq!(Tokens::free_balance(DNAR, &BOB), 950);
		assert_eq!(Tokens::free_balance(DNAR, &CAROL), 1000);

		let bob_ref_count_1 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_1, bob_ref_count_0 + 1);
		let carol_ref_count_0 = System::consumers(&CAROL);

		assert_noop!(
			AuctionManagerModule::serplus_auction_bid_handler(2, 0, (CAROL, 51), Some((BOB, 50))),
			Error::<Runtime>::InvalidBidPrice,
		);
		assert_eq!(
			AuctionManagerModule::serplus_auction_bid_handler(2, 0, (CAROL, 55), Some((BOB, 50))).is_ok(),
			true
		);
		assert_eq!(Tokens::free_balance(DNAR, &BOB), 1000);
		assert_eq!(Tokens::free_balance(DNAR, &CAROL), 945);
		let bob_ref_count_2 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_2, bob_ref_count_1 - 1);
		let carol_ref_count_1 = System::consumers(&CAROL);
		assert_eq!(carol_ref_count_1, carol_ref_count_0 + 1);
	});
}

#[test]
fn bid_when_soft_cap_for_setter_auction_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AuctionManagerModule::new_setter_auction(&ALICE, BTC, 10, 100));
		assert_eq!(
			AuctionManagerModule::on_new_bid(1, 0, (BOB, 100), None).auction_end_change,
			Change::NewValue(Some(101))
		);
		assert_eq!(
			AuctionManagerModule::on_new_bid(2001, 0, (CAROL, 10), Some((BOB, 5))).accept_bid,
			false,
		);
		assert_eq!(
			AuctionManagerModule::on_new_bid(2001, 0, (CAROL, 15), Some((BOB, 5))).auction_end_change,
			Change::NewValue(Some(2051))
		);
	});
}

#[test]
fn bid_when_soft_cap_for_diamond_auction_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AuctionManagerModule::new_diamond_auction(200, 100));
		assert_eq!(
			AuctionManagerModule::on_new_bid(1, 0, (BOB, 100), None).auction_end_change,
			Change::NewValue(Some(101))
		);
		assert_eq!(
			AuctionManagerModule::on_new_bid(2001, 0, (CAROL, 105), Some((BOB, 100))).accept_bid,
			false
		);
		assert_eq!(
			AuctionManagerModule::on_new_bid(2001, 0, (CAROL, 110), Some((BOB, 100))).auction_end_change,
			Change::NewValue(Some(2051))
		);
	});
}

#[test]
fn bid_when_soft_cap_for_serplus_auction_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AuctionManagerModule::new_serplus_auction(100));
		assert_eq!(
			AuctionManagerModule::on_new_bid(1, 0, (BOB, 100), None).auction_end_change,
			Change::NewValue(Some(101))
		);
		assert_eq!(
			AuctionManagerModule::on_new_bid(2001, 0, (CAROL, 105), Some((BOB, 100))).accept_bid,
			false
		);
		assert_eq!(
			AuctionManagerModule::on_new_bid(2001, 0, (CAROL, 110), Some((BOB, 100))).auction_end_change,
			Change::NewValue(Some(2051))
		);
	});
}

#[test]
fn setter_auction_end_handler_without_bid() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SerpTreasuryModule::deposit_reserve(&CAROL, BTC, 100));
		assert_ok!(AuctionManagerModule::new_setter_auction(&ALICE, BTC, 100, 200));
		assert_eq!(SerpTreasuryModule::total_reserves(BTC), 100);
		assert_eq!(AuctionManagerModule::total_target_in_auction(), 200);
		assert_eq!(AuctionManagerModule::total_reserve_in_auction(BTC), 100);
		let alice_ref_count_0 = System::consumers(&ALICE);

		assert_eq!(AuctionManagerModule::setter_auctions(0).is_some(), true);
		AuctionManagerModule::on_auction_ended(0, None);
		let auction_passed_event = Event::auction_manager(crate::Event::CancelAuction(0));
		assert!(System::events()
			.iter()
			.any(|record| record.event == auction_passed_event));

		assert_eq!(SerpTreasuryModule::total_reserves(BTC), 100);
		assert_eq!(AuctionManagerModule::setter_auctions(0), None);
		assert_eq!(AuctionManagerModule::total_target_in_auction(), 0);
		assert_eq!(AuctionManagerModule::total_reserve_in_auction(BTC), 0);
		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 - 1);
	});
}

#[test]
fn setter_auction_end_handler_in_reverse_stage() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SerpTreasuryModule::deposit_reserve(&CAROL, BTC, 100));
		assert_ok!(AuctionManagerModule::new_setter_auction(&ALICE, BTC, 100, 200));
		assert_eq!(
			AuctionManagerModule::setter_auction_bid_handler(2, 0, (BOB, 400), None).is_ok(),
			true
		);
		assert_eq!(SerpTreasuryModule::total_reserves(BTC), 50);
		assert_eq!(AuctionManagerModule::total_reserve_in_auction(BTC), 50);
		assert_eq!(Tokens::free_balance(BTC, &ALICE), 1050);
		assert_eq!(Tokens::free_balance(BTC, &BOB), 1000);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 800);
		assert_eq!(SerpTreasuryModule::serpluspool(), 200);

		let alice_ref_count_0 = System::consumers(&ALICE);
		let bob_ref_count_0 = System::consumers(&BOB);

		assert_eq!(AuctionManagerModule::setter_auctions(0).is_some(), true);
		AuctionManagerModule::on_auction_ended(0, Some((BOB, 400)));
		let auction_dealt_event = Event::auction_manager(crate::Event::SetterAuctionDealt(0, BTC, 50, BOB, 200));
		assert!(System::events()
			.iter()
			.any(|record| record.event == auction_dealt_event));

		assert_eq!(SerpTreasuryModule::total_reserves(BTC), 0);
		assert_eq!(AuctionManagerModule::setter_auctions(0), None);
		assert_eq!(AuctionManagerModule::total_reserve_in_auction(BTC), 0);
		assert_eq!(Tokens::free_balance(BTC, &ALICE), 1050);
		assert_eq!(Tokens::free_balance(BTC, &BOB), 1050);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 800);
		assert_eq!(SerpTreasuryModule::serpluspool(), 200);

		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 - 1);
		let bob_ref_count_1 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_1, bob_ref_count_0 - 1);
	});
}

#[test]
fn setter_auction_end_handler_by_dealing_which_target_not_zero() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SerpTreasuryModule::deposit_reserve(&CAROL, BTC, 100));
		assert_ok!(AuctionManagerModule::new_setter_auction(&ALICE, BTC, 100, 200));
		assert_eq!(
			AuctionManagerModule::setter_auction_bid_handler(1, 0, (BOB, 100), None).is_ok(),
			true
		);
		assert_eq!(SerpTreasuryModule::total_reserves(BTC), 100);
		assert_eq!(AuctionManagerModule::total_target_in_auction(), 200);
		assert_eq!(AuctionManagerModule::total_reserve_in_auction(BTC), 100);
		assert_eq!(Tokens::free_balance(BTC, &BOB), 1000);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 900);
		assert_eq!(SerpTreasuryModule::serpluspool(), 100);

		let alice_ref_count_0 = System::consumers(&ALICE);
		let bob_ref_count_0 = System::consumers(&BOB);

		assert_eq!(AuctionManagerModule::setter_auctions(0).is_some(), true);
		AuctionManagerModule::on_auction_ended(0, Some((BOB, 100)));
		let auction_dealt_event = Event::auction_manager(crate::Event::SetterAuctionDealt(0, BTC, 100, BOB, 100));
		assert!(System::events()
			.iter()
			.any(|record| record.event == auction_dealt_event));

		assert_eq!(SerpTreasuryModule::total_reserves(BTC), 0);
		assert_eq!(AuctionManagerModule::setter_auctions(0), None);
		assert_eq!(AuctionManagerModule::total_target_in_auction(), 0);
		assert_eq!(AuctionManagerModule::total_reserve_in_auction(BTC), 0);
		assert_eq!(AuctionManagerModule::total_target_in_auction(), 0);
		assert_eq!(Tokens::free_balance(BTC, &BOB), 1100);

		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 - 1);
		let bob_ref_count_1 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_1, bob_ref_count_0 - 1);
	});
}

#[test]
fn setter_auction_end_handler_by_dex_which_target_not_zero() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SerpTreasuryModule::deposit_reserve(&CAROL, BTC, 100));
		assert_ok!(AuctionManagerModule::new_setter_auction(&ALICE, BTC, 100, 200));
		assert_eq!(
			AuctionManagerModule::setter_auction_bid_handler(1, 0, (BOB, 20), None).is_ok(),
			true
		);
		assert_ok!(DEXModule::add_liquidity(
			Origin::signed(CAROL),
			BTC,
			USDJ,
			100,
			1000,
			false
		));
		assert_eq!(DEXModule::get_swap_target_amount(&[BTC, USDJ], 100, None).unwrap(), 500);

		assert_eq!(SerpTreasuryModule::total_reserves(BTC), 100);
		assert_eq!(AuctionManagerModule::total_target_in_auction(), 200);
		assert_eq!(AuctionManagerModule::total_reserve_in_auction(BTC), 100);
		assert_eq!(Tokens::free_balance(BTC, &BOB), 1000);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 980);
		assert_eq!(Tokens::free_balance(USDJ, &ALICE), 1000);
		assert_eq!(SerpTreasuryModule::standard_pool(), 0);
		assert_eq!(SerpTreasuryModule::serpluspool(), 20);

		let alice_ref_count_0 = System::consumers(&ALICE);
		let bob_ref_count_0 = System::consumers(&BOB);

		assert_eq!(AuctionManagerModule::setter_auctions(0).is_some(), true);
		AuctionManagerModule::on_auction_ended(0, Some((BOB, 20)));
		let dex_take_setter_auction =
			Event::auction_manager(crate::Event::DEXTakeSetterAuction(0, BTC, 100, 500));
		assert!(System::events()
			.iter()
			.any(|record| record.event == dex_take_setter_auction));

		assert_eq!(SerpTreasuryModule::total_reserves(BTC), 0);
		assert_eq!(AuctionManagerModule::setter_auctions(0), None);
		assert_eq!(AuctionManagerModule::total_target_in_auction(), 0);
		assert_eq!(AuctionManagerModule::total_reserve_in_auction(BTC), 0);
		assert_eq!(Tokens::free_balance(BTC, &BOB), 1000);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 1000);
		assert_eq!(Tokens::free_balance(USDJ, &ALICE), 1300);
		assert_eq!(SerpTreasuryModule::standard_pool(), 320);
		assert_eq!(SerpTreasuryModule::serpluspool(), 520);

		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 - 1);
		let bob_ref_count_1 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_1, bob_ref_count_0 - 1);
	});
}

#[test]
fn diamond_auction_end_handler_without_bid() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(AuctionManagerModule::new_diamond_auction(300, 100));
		assert_eq!(AuctionManagerModule::total_standard_in_auction(), 100);

		assert_eq!(AuctionManagerModule::diamond_auctions(0).is_some(), true);
		AuctionManagerModule::on_auction_ended(0, None);
		let auction_passed_event = Event::auction_manager(crate::Event::CancelAuction(0));
		assert!(System::events()
			.iter()
			.any(|record| record.event == auction_passed_event));

		assert_eq!(AuctionManagerModule::diamond_auctions(0), None);
		assert_eq!(AuctionManagerModule::total_standard_in_auction(), 0);
	});
}

#[test]
fn diamond_auction_end_handler_with_bid() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(AuctionManagerModule::new_diamond_auction(300, 100));
		assert_eq!(
			AuctionManagerModule::diamond_auction_bid_handler(1, 0, (BOB, 100), None).is_ok(),
			true
		);
		assert_eq!(AuctionManagerModule::total_standard_in_auction(), 100);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 900);
		assert_eq!(Tokens::free_balance(DNAR, &BOB), 1000);

		let bob_ref_count_0 = System::consumers(&BOB);

		assert_eq!(AuctionManagerModule::diamond_auctions(0).is_some(), true);
		AuctionManagerModule::on_auction_ended(0, Some((BOB, 100)));
		let diamond_auction_deal_event = Event::auction_manager(crate::Event::DiamondAuctionDealt(0, 300, BOB, 100));
		assert!(System::events()
			.iter()
			.any(|record| record.event == diamond_auction_deal_event));

		assert_eq!(Tokens::free_balance(DNAR, &BOB), 1300);
		assert_eq!(Tokens::total_issuance(DNAR), 3300);
		assert_eq!(AuctionManagerModule::diamond_auctions(0), None);
		assert_eq!(AuctionManagerModule::total_standard_in_auction(), 0);

		let bob_ref_count_1 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_1, bob_ref_count_0 - 1);
	});
}

#[test]
fn serplus_auction_end_handler_without_bid() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(AuctionManagerModule::new_serplus_auction(100));
		assert_eq!(AuctionManagerModule::total_serplusin_auction(), 100);

		assert_eq!(AuctionManagerModule::serplus_auctions(0).is_some(), true);
		AuctionManagerModule::on_auction_ended(0, None);
		let auction_passed_event = Event::auction_manager(crate::Event::CancelAuction(0));
		assert!(System::events()
			.iter()
			.any(|record| record.event == auction_passed_event));

		assert_eq!(AuctionManagerModule::serplus_auctions(0), None);
		assert_eq!(AuctionManagerModule::total_serplusin_auction(), 0);
	});
}

#[test]
fn serplus_auction_end_handler_with_bid() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(SerpTreasuryModule::on_system_serplus(100));
		assert_ok!(AuctionManagerModule::new_serplus_auction(100));
		assert_eq!(
			AuctionManagerModule::serplus_auction_bid_handler(1, 0, (BOB, 500), None).is_ok(),
			true
		);
		assert_eq!(SerpTreasuryModule::standard_pool(), 0);
		assert_eq!(AuctionManagerModule::total_serplusin_auction(), 100);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 1000);
		assert_eq!(Tokens::free_balance(DNAR, &BOB), 500);
		assert_eq!(Tokens::total_issuance(DNAR), 2500);

		let bob_ref_count_0 = System::consumers(&BOB);

		assert_eq!(AuctionManagerModule::serplus_auctions(0).is_some(), true);
		AuctionManagerModule::on_auction_ended(0, Some((BOB, 500)));
		let serplus_auction_deal_event = Event::auction_manager(crate::Event::SerplusAuctionDealt(0, 100, BOB, 500));
		assert!(System::events()
			.iter()
			.any(|record| record.event == serplus_auction_deal_event));

		assert_eq!(SerpTreasuryModule::standard_pool(), 100);
		assert_eq!(AuctionManagerModule::serplus_auctions(0), None);
		assert_eq!(AuctionManagerModule::total_serplusin_auction(), 0);
		assert_eq!(Tokens::free_balance(USDJ, &BOB), 1100);
		assert_eq!(Tokens::total_issuance(DNAR), 2500);

		let bob_ref_count_1 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_1, bob_ref_count_0 - 1);
	});
}

#[test]
fn swap_bidders_works() {
	ExtBuilder::default().build().execute_with(|| {
		let alice_ref_count_0 = System::consumers(&ALICE);
		let bob_ref_count_0 = System::consumers(&BOB);

		AuctionManagerModule::swap_bidders(&BOB, None);

		let bob_ref_count_1 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_1, bob_ref_count_0 + 1);

		AuctionManagerModule::swap_bidders(&ALICE, Some(&BOB));

		let alice_ref_count_1 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_1, alice_ref_count_0 + 1);
		let bob_ref_count_2 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_2, bob_ref_count_1 - 1);

		AuctionManagerModule::swap_bidders(&BOB, Some(&ALICE));

		let alice_ref_count_2 = System::consumers(&ALICE);
		assert_eq!(alice_ref_count_2, alice_ref_count_1 - 1);
		let bob_ref_count_3 = System::consumers(&BOB);
		assert_eq!(bob_ref_count_3, bob_ref_count_2 + 1);
	});
}
