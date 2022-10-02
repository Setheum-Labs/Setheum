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

//! Unit tests for the airdrop module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Airdrop, Event, ExtBuilder, Origin, System, SETR, ALICE, BOB, CHARLIE, DAVE, EVE, TREASURY, SETUSD};
use sp_runtime::traits::BadOrigin;

#[test]
fn fund_airdrop_treasury_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(Airdrop::fund_airdrop_treasury(
            Origin::signed(BOB),
            SETUSD, 10
        ),
        BadOrigin
        );
        
        let airdrop_treasury = Airdrop::account_id();

        assert_ok!(Airdrop::fund_airdrop_treasury(Origin::signed(ALICE), SETR, 258));
        System::assert_last_event(Event::AirDrop(
            crate::Event::FundAirdropTreasury {
                funder: ALICE,
                currency_id: SETR,
                amount: 258
            },
        ));
        assert_eq!(Tokens::free_balance(SETUSD, airdrop_treasury), 0);
        assert_eq!(Tokens::free_balance(SETR, airdrop_treasury), 258);


        assert_ok!(Airdrop::fund_airdrop_treasury(Origin::signed(ALICE), SETR, 10));
        System::assert_last_event(Event::AirDrop(
            crate::Event::FundAirdropTreasury {
                funder: ALICE,
                currency_id: SETR,
                amount: 10
            },
        ));
         assert_eq!(Tokens::free_balance(SETR, airdrop_treasury), 268);

        assert_ok!(Airdrop::fund_airdrop_treasury(Origin::signed(ALICE), SETUSD, 258));
        System::assert_last_event(Event::AirDrop(
            crate::Event::FundAirdropTreasury {
                funder: ALICE,
                currency_id: SETUSD,
                amount: 258
            },
        ));
        assert_eq!(Tokens::free_balance(SETUSD, airdrop_treasury), 258);
	});
}

#[test]
fn make_airdrop_works() {
	ExtBuilder::default().build().execute_with(|| {
        let airdrop_list = vec![
            (ALICE, 10),
            (BOB, 5),
            (CHARLIE, 20),
        ];

		assert_noop!(Airdrop::make_airdrop(
            Origin::signed(BOB),
            SETUSD,
            airdrop_list
        ),
        BadOrigin
        );

        assert_ok!(Airdrop::fund_airdrop_treasury(Origin::signed(ALICE), SETR, 258));
        System::assert_last_event(Event::AirDrop(
            crate::Event::FundAirdropTreasury {
                funder: ALICE,
                currency_id: SETR,
                amount: 258
            },
        ));
        assert_eq!(Tokens::free_balance(SETR, airdrop_treasury), 258);

        assert_ok!(Airdrop::fund_airdrop_treasury(Origin::signed(ALICE), SETUSD, 258));
        System::assert_last_event(Event::AirDrop(
            crate::Event::FundAirdropTreasury {
                funder: ALICE,
                currency_id: SETUSD,
                amount: 258
            },
        ));
        assert_eq!(Tokens::free_balance(SETUSD, airdrop_treasury), 258);

        assert_ok!(Airdrop::make_airdrop(
            Origin::signed(ALICE),
            SETR,
            airdrop_list
        ));
        System::assert_last_event(Event::AirDrop(
            crate::Event::Airdrop {
                currency_id: SETR,
                airdrop_list
            },
        ));
        assert_ok!(Airdrop::make_airdrop(
            Origin::signed(ALICE),
            SETUSD,
            airdrop_list
        ));
        System::assert_last_event(Event::AirDrop(
            crate::Event::Airdrop {
                currency_id: SETUSD,
                airdrop_list
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
            (CHARLIE, 20),
            (DAVE, 20),
            (EVE, 20),
        ];

		assert_noop!(Airdrop::make_airdrop(
            Origin::signed(BOB),
            SETUSD,
            airdrop_list
        ),
        BadOrigin
        );

        assert_ok!(Airdrop::fund_airdrop_treasury(Origin::signed(ALICE), SETR, 258));
        System::assert_last_event(Event::AirDrop(
            crate::Event::FundAirdropTreasury {
                funder: ALICE,
                currency_id: SETR,
                amount: 258
            },
        ));
        assert_eq!(Tokens::free_balance(SETR, airdrop_treasury), 258);

        assert_noop!(Airdrop::make_airdrop(
            Origin::signed(ALICE),
            SETR,
            airdrop_list
        )
        Error::<Runtime>::OverSizedAirdropList,
        );
	});
}
