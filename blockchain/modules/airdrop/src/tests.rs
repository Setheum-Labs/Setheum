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

//! Unit tests for the airdrop module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_runtime::traits::BadOrigin;

#[test]
fn make_airdrop_works() {
    ExtBuilder::default().build().execute_with(|| {
        let airdrop_list = vec![
            (ALICE, 10),
            (BOB, 5),
        ];

        assert_ok!(Airdrop::make_airdrop(Origin::signed(ALICE), SETR, airdrop_list.clone()));
        System::assert_last_event(Event::AirDrop(
            crate::Event::Airdrop {
                currency_id: SETR,
                airdrop_list: airdrop_list.clone()
            },
        ));
        assert_ok!(Airdrop::make_airdrop(Origin::signed(ALICE), SETUSD, airdrop_list));
        System::assert_last_event(Event::AirDrop(
            crate::Event::Airdrop {
                currency_id: SETUSD,
                airdrop_list
            },
        ));
    });
}

#[test]
fn make_airdrop_with_json_works() {
    ExtBuilder::default().build().execute_with(|| {
        let valid_json = br#"
        [
            {"account": "ALICE", "amount": 10},
            {"account": "BOB", "amount": 5}
        ]
        "#.as_bytes().to_vec();

        assert_ok!(Airdrop::make_airdrop_with_json(Origin::signed(ALICE), SETR, valid_json.clone()));
        System::assert_last_event(Event::AirDrop(
            crate::Event::Airdrop {
                currency_id: SETR,
                airdrop_list: vec![
                    (ALICE, 10),
                    (BOB, 5),
                ]
            },
        ));
        assert_ok!(Airdrop::make_airdrop_with_json(Origin::signed(ALICE), SETUSD, valid_json));
        System::assert_last_event(Event::AirDrop(
            crate::Event::Airdrop {
                currency_id: SETUSD,
                airdrop_list: vec![
                    (ALICE, 10),
                    (BOB, 5),
                ]
            },
        ));
    });
}

#[test]
fn make_airdrop_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let airdrop_list = vec![
            (ALICE, 10),
            (BOB, 5),
        ];

        assert_ok!(Airdrop::make_airdrop(Origin::signed(ALICE), SETR, airdrop_list.clone()));
        System::assert_last_event(Event::AirDrop(
            crate::Event::Airdrop {
                currency_id: SETR,
                airdrop_list: airdrop_list.clone()
            },
        ));
        assert_eq!(Tokens::free_balance(SETR, Airdrop::account_id()), 258);

        assert_noop!(
            Airdrop::make_airdrop(Origin::signed(ALICE), SETR, airdrop_list),
            Error::<Runtime>::OverSizedAirdropList,
        );
    });
}

#[test]
fn make_airdrop_with_json_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let oversized_json = br#"
        [
            {"account": "ALICE", "amount": 10},
            {"account": "BOB", "amount": 5}
        ]
        "#.as_bytes().to_vec();

        assert_ok!(Airdrop::make_airdrop_with_json(Origin::signed(ALICE), SETR, oversized_json.clone()));
        System::assert_last_event(Event::AirDrop(
            crate::Event::Airdrop {
                currency_id: SETR,
                airdrop_list: vec![
                    (ALICE, 10),
                    (BOB, 5),
                ]
            },
        ));
        assert_eq!(Tokens::free_balance(SETR, Airdrop::account_id()), 258);

        assert_noop!(
            Airdrop::make_airdrop_with_json(Origin::signed(ALICE), SETR, oversized_json),
            Error::<Runtime>::OverSizedAirdropList
        );
    });
}
