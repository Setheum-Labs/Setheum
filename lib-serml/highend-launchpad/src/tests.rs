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
use frame_support::assert_ok;
use mock::*;

#[test]
fn make_proposal_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(LaunchPadCrowdsales::make_proposal(
            Origin::signed(ALICE),
            "Project Name".to_owned(),
            "Project Logo".to_owned(),
            "Project Description".to_owned(),
            "project.website".to_owned(),
            BOB,
            SETUSD,
            TEST,
            10,
            10_000,
            100_000,
            20
        ));
        assert_eq!(
            LaunchPadCrowdsales::proposal_info(0),
            Some(ProposalInfo {
                origin: ALICE,
                project_name: "Project Name".to_owned(),
                project_logo: "Project Logo".to_owned(),
                project_description: "Project Description".to_owned(),
                project_website: "project.website".to_owned(),
                beneficiary: BOB,
                pool: TREASURY,
                raise_currency: SETUSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: vec![],
                proposal_time: 0,
                period: 20,
                campaign_start: 0,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: false,
                is_active: false,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            })
        )
    });
}
#[test]
fn proposal_info_works() {
	ExtBuilder::default().build().execute_with(|| {
        assert_ok!(LaunchPadCrowdsales::make_proposal(
            Origin::signed(ALICE),
            "Project Name".to_owned(),
            "Project Logo".to_owned(),
            "Project Description".to_owned(),
            "project.website".to_owned(),
            BOB,
            SETUSD,
            TEST,
            10,
            10_000,
            100_000,
            20
        ));
        assert_eq!(
            LaunchPadCrowdsales::proposal_info(0),
            Some(ProposalInfo {
                origin: ALICE,
                project_name: "Project Name".to_owned(),
                project_logo: "Project Logo".to_owned(),
                project_description: "Project Description".to_owned(),
                project_website: "project.website".to_owned(),
                beneficiary: BOB,
                pool: TREASURY,
                raise_currency: SETUSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: vec![],
                proposal_time: 0,
                period: 20,
                campaign_start: 0,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: false,
                is_active: false,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            })
        )
    });
}

#[test]
fn proposal_info_none_works() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            LaunchPadCrowdsales::proposal_info(0),
            None
        )
    });
}

#[test]
fn proposal_info_works_for_proposal_id_1() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(LaunchPadCrowdsales::make_proposal(
            Origin::signed(ALICE),
            "Project Name".to_owned(),
            "Project Logo".to_owned(),
            "Project Description".to_owned(),
            "project.website".to_owned(),
            BOB,
            SETUSD,
            TEST,
            10,
            10_000,
            100_000,
            20
        ));
        assert_ok!(LaunchPadCrowdsales::make_proposal(
            Origin::signed(ALICE),
            "Project 1 Name".to_owned(),
            "Project 1 Logo".to_owned(),
            "Project 1 Description".to_owned(),
            "project1.website".to_owned(),
            BOB,
            SETUSD,
            TEST,
            11,
            11_000,
            110_000,
            21
        ));
        assert_eq!(
            LaunchPadCrowdsales::proposal_info(1),
            Some(ProposalInfo {
                origin: ALICE,
                project_name: "Project 1 Name".to_owned(),
                project_logo: "Project 1 Logo".to_owned(),
                project_description: "Project 1 Description".to_owned(),
                project_website: "project1.website".to_owned(),
                beneficiary: BOB,
                pool: TREASURY,
                raise_currency: SETUSD,
                sale_token: TEST,
                token_price: 11,
                crowd_allocation: 11_000,
                goal: 110_000,
                raised: 0,
                contributors_count: 0,
                contributions: vec![],
                proposal_time: 0,
                period: 21,
                campaign_start: 0,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: false,
                is_active: false,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            })
        )
    });
}

#[test]
fn new_proposal_works() {
    ExtBuilder::Default().build().execute_with(|| {
        assert_ok!(LaunchPadCrowdsales::new_proposal(
            ALICE,
            "Project Name".to_owned(),
            "Project Logo".to_owned(),
            "Project Description".to_owned(),
            "project.website".to_owned(),
            BOB,
            SETUSD,
            TEST,
            10,
            10_000,
            100_000,
            20
        ));
        assert_eq!(
            LaunchPadCrowdsales::proposal_info(0),
            Some(ProposalInfo {
                origin: ALICE,
                project_name: "Project Name".to_owned(),
                project_logo: "Project Logo".to_owned(),
                project_description: "Project Description".to_owned(),
                project_website: "project.website".to_owned(),
                beneficiary: BOB,
                pool: TREASURY,
                raise_currency: SETUSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: vec![],
                proposal_time: 0,
                period: 20,
                campaign_start: 0,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: false,
                is_active: false,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            })
        )
        ))
    })
}

#[test]
fn ensure_valid_proposal_works() {
    ExtBuilder::Default().build().execute_with(|| {
        assert_ok!(LaunchPadCrowdsales::new_proposal(
            Origin::signed(ALICE),
            "Project Name".to_owned(),
            "Project Logo".to_owned(),
            "Project Description".to_owned(),
            "project.website".to_owned(),
            BOB,
            SETUSD,
            TEST,
            10,
            10_000,
            100_000,
            20
        ));
        assert_ok!(LaunchPadCrowdsales::ensure_valid_proposal(0));
        assert_noop!(
            LaunchPadCrowdsales::ensure_valid_proposal(1),
            Error::<Runtime>::ProposalNotFound
        );
    })
}

#[test]
fn ensure_valid_proposal_not_works_for_approved_proposal() {
    ExtBuilder::Default().build().execute_with(|| {
        assert_ok!(LaunchPadCrowdsales::new_proposal(
            Origin::signed(ALICE),
            "Project Name".to_owned(),
            "Project Logo".to_owned(),
            "Project Description".to_owned(),
            "project.website".to_owned(),
            BOB,
            SETUSD,
            TEST,
            10,
            10_000,
            100_000,
            20
        ));
        assert_ok!(LaunchPadCrowdsales::approve_proposal(0));
        assert_noop!(
            LaunchPadCrowdsales::ensure_valid_proposal(Origin::signed(ALICE), 0),
            Error::<Runtime>::ProposalAlreadyApproved
        );
    })
}
#[test]
fn_approve_proposal_works() {
    ExtBuilder::Default().build().execute_with(|| {
        assert_ok!(LaunchPadCrowdsales::new_proposal(
            Origin::signed(ALICE),
            "Project Name".to_owned(),
            "Project Logo".to_owned(),
            "Project Description".to_owned(),
            "project.website".to_owned(),
            BOB,
            SETUSD,
            TEST,
            10,
            10_000,
            100_000,
            20
        ));
        assert_ok!(LaunchPadCrowdsales::approve_proposal(0));
        assert_eq!(
            LaunchPadCrowdsales::proposal_info(0),
            Some(ProposalInfo {
                origin: ALICE,
                project_name: "Project Name".to_owned(),
                project_logo: "Project Logo".to_owned(),
                project_description: "Project Description".to_owned(),
                project_website: "project.website".to_owned(),
                beneficiary: BOB,
                pool: BOB,
                raise_currency: SETUSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 1,
                contributions: vec![],
                proposal_time: 0,
                period: 20,
                campaign_start: 0,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: false,
                is_active: false,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            })
        )
    })
}

// Reject proposal works
#[test]
fn reject_proposal_works() {
    ExtBuilder::Default().build().execute_with(|| {
        assert_ok!(LaunchPadCrowdsales::new_proposal(
            Origin::signed(ALICE),
            "Project Name".to_owned(),
            "Project Logo".to_owned(),
            "Project Description".to_owned(),
            "project.website".to_owned(),
            BOB,
            SETUSD,
            TEST,
            10,
            10_000,
            100_000,
            20
        ));
        assert_ok!(LaunchPadCrowdsales::reject_proposal(0));
        assert_eq!(
            LaunchPadCrowdsales::proposal_info(0),
            Some(ProposalInfo {
                origin: ALICE,
                project_name: "Project Name".to_owned(),
                project_logo: "Project Logo".to_owned(),
                project_description: "Project Description".to_owned(),
                project_website: "project.website".to_owned(),
                beneficiary: BOB,
                pool: BOB,
                raise_currency: SETUSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: vec![],
                proposal_time: 0,
                period: 20,
                campaign_start: 0,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: true,
                is_waiting: false,
                is_active: false,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            })
        )
    })
}