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

//! Unit tests for the chainbridge module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn register_resource_id_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_eq!(SetheumChainBridge::resource_ids(DNAR::get()), None);
		assert_eq!(SetheumChainBridge::currency_ids(DNARResourceId::get()), None);

		assert_noop!(
			SetheumChainBridge::register_resource_id(Origin::signed(ALICE), DNARResourceId::get(), DNAR::get()),
			BadOrigin,
		);

		assert_noop!(
			SetheumChainBridge::register_resource_id(
				Origin::signed(RegistorOrigin::get()),
				DNARResourceId::get(),
				WETH::get()
			),
			Error::<Runtime>::ResourceIdCurrencyIdNotMatch,
		);

		assert_noop!(
			SetheumChainBridge::register_resource_id(
				Origin::signed(RegistorOrigin::get()),
				WETHResourceId::get(),
				DNAR::get()
			),
			Error::<Runtime>::ResourceIdCurrencyIdNotMatch,
		);

		assert_ok!(SetheumChainBridge::register_resource_id(
			Origin::signed(RegistorOrigin::get()),
			DNARResourceId::get(),
			DNAR::get()
		));
		let register_event =
			Event::setheum_chainbridge(crate::Event::RegisterResourceId(DNARResourceId::get(), DNAR::get()));
		assert!(System::events().iter().any(|record| record.event == register_event));

		assert_eq!(SetheumChainBridge::resource_ids(DNAR::get()), Some(DNARResourceId::get()));
		assert_eq!(SetheumChainBridge::currency_ids(DNARResourceId::get()), Some(DNAR::get()));

		assert_noop!(
			SetheumChainBridge::register_resource_id(
				Origin::signed(RegistorOrigin::get()),
				DNARResourceId::get(),
				DNAR::get()
			),
			Error::<Runtime>::ResourceIdAlreadyRegistered,
		);
	});
}

#[test]
fn remove_resource_id_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(SetheumChainBridge::register_resource_id(
			Origin::signed(RegistorOrigin::get()),
			DNARResourceId::get(),
			DNAR::get()
		));
		assert_eq!(SetheumChainBridge::resource_ids(DNAR::get()), Some(DNARResourceId::get()));
		assert_eq!(SetheumChainBridge::currency_ids(DNARResourceId::get()), Some(DNAR::get()));

		assert_noop!(
			SetheumChainBridge::remove_resource_id(Origin::signed(ALICE), DNARResourceId::get()),
			BadOrigin,
		);

		assert_ok!(SetheumChainBridge::remove_resource_id(
			Origin::signed(RegistorOrigin::get()),
			DNARResourceId::get()
		));
		let unregister_event =
			Event::setheum_chainbridge(crate::Event::UnregisterResourceId(DNARResourceId::get(), DNAR::get()));
		assert!(System::events().iter().any(|record| record.event == unregister_event));
	});
}

#[test]
fn is_origin_chain_resource_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(SetheumChainBridge::is_origin_chain_resource(DNARResourceId::get()), true);
		assert_eq!(
			SetheumChainBridge::is_origin_chain_resource(WETHResourceId::get()),
			false
		);
	});
}

#[test]
fn do_transfer_to_bridge_work() {
	ExtBuilder::default().build().execute_with(|| {
		let dest_chain_id: chainbridge::ChainId = 0;

		assert_noop!(
			SetheumChainBridge::do_transfer_to_bridge(&ALICE, DNAR::get(), dest_chain_id, vec![1], 10),
			Error::<Runtime>::InvalidDestChainId,
		);

		assert_ok!(ChainBridge::whitelist_chain(
			Origin::signed(AdminOrigin::get()),
			dest_chain_id
		));
		assert_noop!(
			SetheumChainBridge::do_transfer_to_bridge(&ALICE, DNAR::get(), dest_chain_id, vec![1], 10),
			Error::<Runtime>::ResourceIdNotRegistered,
		);

		assert_ok!(SetheumChainBridge::register_resource_id(
			Origin::signed(RegistorOrigin::get()),
			DNARResourceId::get(),
			DNAR::get()
		));
		assert_eq!(Tokens::total_issuance(DNAR::get()), 1000);
		assert_eq!(Tokens::free_balance(DNAR::get(), &ALICE), 1000);
		assert_eq!(Tokens::free_balance(DNAR::get(), &ChainBridge::account_id()), 0);

		assert_ok!(SetheumChainBridge::do_transfer_to_bridge(
			&ALICE,
			DNAR::get(),
			dest_chain_id,
			vec![1],
			10
		));
		assert_eq!(Tokens::total_issuance(DNAR::get()), 1000);
		assert_eq!(Tokens::free_balance(DNAR::get(), &ALICE), 990);
		assert_eq!(Tokens::free_balance(DNAR::get(), &ChainBridge::account_id()), 10);

		assert_ok!(SetheumChainBridge::register_resource_id(
			Origin::signed(RegistorOrigin::get()),
			WETHResourceId::get(),
			WETH::get()
		));
		assert_ok!(Tokens::deposit(WETH::get(), &ALICE, 1000));
		assert_eq!(Tokens::total_issuance(WETH::get()), 1000);
		assert_eq!(Tokens::free_balance(WETH::get(), &ALICE), 1000);
		assert_eq!(Tokens::free_balance(WETH::get(), &ChainBridge::account_id()), 0);

		assert_ok!(SetheumChainBridge::do_transfer_to_bridge(
			&ALICE,
			WETH::get(),
			dest_chain_id,
			vec![1],
			20
		));
		assert_eq!(Tokens::total_issuance(WETH::get()), 980);
		assert_eq!(Tokens::free_balance(WETH::get(), &ALICE), 980);
		assert_eq!(Tokens::free_balance(WETH::get(), &ChainBridge::account_id()), 0);
	});
}

#[test]
fn transfer_from_bridge_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			SetheumChainBridge::transfer_from_bridge(Origin::signed(ALICE), ALICE, 500, DNARResourceId::get()),
			BadOrigin,
		);

		assert_noop!(
			SetheumChainBridge::transfer_from_bridge(
				Origin::signed(ChainBridge::account_id()),
				ALICE,
				500,
				DNARResourceId::get()
			),
			Error::<Runtime>::ResourceIdNotRegistered,
		);

		assert_ok!(SetheumChainBridge::register_resource_id(
			Origin::signed(RegistorOrigin::get()),
			DNARResourceId::get(),
			DNAR::get()
		));
		assert_ok!(Tokens::deposit(DNAR::get(), &ChainBridge::account_id(), 1000));
		assert_eq!(Tokens::total_issuance(DNAR::get()), 2000);
		assert_eq!(Tokens::free_balance(DNAR::get(), &ALICE), 1000);
		assert_eq!(Tokens::free_balance(DNAR::get(), &ChainBridge::account_id()), 1000);

		assert_ok!(SetheumChainBridge::transfer_from_bridge(
			Origin::signed(ChainBridge::account_id()),
			ALICE,
			500,
			DNARResourceId::get()
		));
		assert_eq!(Tokens::total_issuance(DNAR::get()), 2000);
		assert_eq!(Tokens::free_balance(DNAR::get(), &ALICE), 1500);
		assert_eq!(Tokens::free_balance(DNAR::get(), &ChainBridge::account_id()), 500);

		assert_ok!(SetheumChainBridge::register_resource_id(
			Origin::signed(RegistorOrigin::get()),
			WETHResourceId::get(),
			WETH::get()
		));
		assert_eq!(Tokens::total_issuance(WETH::get()), 0);
		assert_eq!(Tokens::free_balance(WETH::get(), &ALICE), 0);
		assert_eq!(Tokens::free_balance(WETH::get(), &ChainBridge::account_id()), 0);

		assert_ok!(SetheumChainBridge::transfer_from_bridge(
			Origin::signed(ChainBridge::account_id()),
			ALICE,
			500,
			WETHResourceId::get()
		));
		assert_eq!(Tokens::total_issuance(WETH::get()), 500);
		assert_eq!(Tokens::free_balance(WETH::get(), &ALICE), 500);
		assert_eq!(Tokens::free_balance(WETH::get(), &ChainBridge::account_id()), 0);
	});
}
