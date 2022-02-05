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

//! Unit tests for the Launchpad Crowdsales Pallet.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn proposal_info_works() {
	ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            assert_ok!(LaunchPad::new_proposal(
                ALICE,
                "Project Name".as_bytes().to_vec(),
                "Project Logo".as_bytes().to_vec(),
                "Project Description".as_bytes().to_vec(),
                "Project Website".as_bytes().to_vec(),
                BOB,
                SETUSD,
                TEST,
                10,
                10_000,
                100_000,
                20
            ));
        });
}

#[test]
fn make_proposal_works() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            assert_ok!(LaunchPad::make_proposal(
                Origin::signed(ALICE),
                "Project Name".as_bytes().to_vec(),
                "Project Logo".as_bytes().to_vec(),
                "Project Description".as_bytes().to_vec(),
                "Project Website".as_bytes().to_vec(),
                BOB,
                SETUSD,
                TEST,
                10,
                10_000,
                100_000,
                20
            ));
        });
}

