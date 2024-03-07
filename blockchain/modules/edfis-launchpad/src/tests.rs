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

//! Unit tests for the Edfis Launchpad module.

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
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
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
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_eq!(
                LaunchPad::proposal_info(TEST),
                Some(CampaignInfo {
                    id:TEST,
                    origin: ALICE,
                    beneficiary: BOB,
                    pool: LaunchPad::campaign_pool(0),
                    raise_currency: USSD,
                    sale_token: TEST,
                    token_price: 10,
                    crowd_allocation: 10_000,
                    goal: 100_000,
                    raised: 0,
                    contributors_count: 0,
                    contributions: vec![],
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
fn campaign_info_works() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 20,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            LaunchPad::on_initialize(23);
            assert_eq!(
                LaunchPad::campaign_info(TEST),
                Some(CampaignInfo {
                    id:TEST,
                    origin: ALICE,
                    beneficiary: BOB,
                    pool: LaunchPad::campaign_pool(0),
                    raise_currency: USSD,
                    sale_token: TEST,
                    token_price: 10,
                    crowd_allocation: 10_000,
                    goal: 100_000,
                    raised: 0,
                    contributors_count: 0,
                    contributions: vec![],
                    period: 20,
                    campaign_start: 20,
                    campaign_end: 40,
                    campaign_retirement_period: 0,
                    proposal_retirement_period: 0,
                    is_approved: true,
                    is_rejected: false,
                    is_waiting: false,
                    is_active: true,
                    is_successful: false,
                    is_failed: false,
                    is_ended: false,
                    is_claimed: false,
                })
            )
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
                BOB,
                USSD,
                TEST,
                10,
                10_000,
                100_000,
                20
            ));
        });
}

#[test]
fn make_proposal_does_not_work() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            assert_noop!(
                LaunchPad::make_proposal(
                    Origin::signed(ALICE),
                    BOB,
                    USSD,
                    TEST,
                    10,
                    10_000,
                    100_000,
                    0
                ),
                Error::<Runtime>::ZeroPeriod
            );
            assert_noop!(
                LaunchPad::make_proposal(
                    Origin::signed(ALICE),
                    BOB,
                    USSD,
                    TEST,
                    10,
                    10_000,
                    100_000,
                    21
                ),
                Error::<Runtime>::MaxActivePeriodExceeded
            );
            assert_noop!(
                LaunchPad::make_proposal(
                    Origin::signed(ALICE),
                    BOB,
                    USSD,
                    TEST,
                    10,
                    10_000,
                    99,
                    20
                ),
                Error::<Runtime>::GoalBelowMinimumRaise
            );
        });
}

#[test]
fn contribute_works() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 21,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            assert_ok!(LaunchPad::contribute(
                Origin::signed(BOB),
                TEST,
                10_000
            ));

            assert_ok!(LaunchPad::contribute(
                Origin::signed(ALICE),
                TEST,
                10_000
            ));
            assert_noop!(
                LaunchPad::contribute(
                    Origin::signed(BOB),
                    TEST,
                    10
                ),
                Error::<Runtime>::ContributionTooSmall
            );
            assert_noop!(
                LaunchPad::contribute(
                    Origin::signed(BOB),
                    TEST,
                    100_001
                ),
                Error::<Runtime>::ContributionCurrencyNotEnough
            );
        });
}

#[test]
fn contribute_does_not_work() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
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
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );

            assert_noop!(
                LaunchPad::contribute(
                    Origin::signed(BOB),
                    TEST,
                    10
                ),
                Error::<Runtime>::CampaignNotFound
            );
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            assert_noop!(
                LaunchPad::contribute(
                    Origin::signed(BOB),
                    TEST,
                    10_000
                ),
                Error::<Runtime>::CampaignNotActive
            );
        });
}

#[test]
fn claim_contribution_allocation_works() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 21,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            assert_ok!(LaunchPad::contribute(
                Origin::signed(BOB),
                TEST,
                50_000
            ));

            assert_ok!(LaunchPad::contribute(
                Origin::signed(ALICE),
                TEST,
                50_000
            ));
           
            LaunchPad::on_initialize(41);
            System::set_block_number(41);
            assert_ok!(LaunchPad::claim_contribution_allocation(
                Origin::signed(BOB),
                TEST,
            ));
        });
}

#[test]
fn claim_contribution_allocation_does_not_work() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
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
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            LaunchPad::on_initialize(23);
            assert_ok!(LaunchPad::contribute(
                Origin::signed(BOB),
                TEST,
                10_000
            ));

            assert_ok!(LaunchPad::contribute(
                Origin::signed(ALICE),
                TEST,
                10_000
            ));

            assert_noop!(
                LaunchPad::claim_contribution_allocation(
                    Origin::signed(BOB),
                    TEST,
                ),
                Error::<Runtime>::CampaignFailed
            );
        });
}

#[test]
fn claim_campaign_fundraise_works() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 21,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            assert_ok!(LaunchPad::contribute(
                Origin::signed(BOB),
                TEST,
                50_000
            ));

            assert_ok!(LaunchPad::contribute(
                Origin::signed(ALICE),
                TEST,
                50_000
            ));
           
            LaunchPad::on_initialize(41);
            System::set_block_number(41);
            assert_ok!(LaunchPad::claim_campaign_fundraise(
                Origin::signed(ALICE),
                TEST,
            ));
        });
}

#[test]
fn claim_campaign_fundraise_does_not_work() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
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
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            LaunchPad::on_initialize(23);
            assert_ok!(LaunchPad::contribute(
                Origin::signed(BOB),
                TEST,
                10_000
            ));

            assert_ok!(LaunchPad::contribute(
                Origin::signed(ALICE),
                TEST,
                10_000
            ));

            LaunchPad::on_initialize(41);
            System::set_block_number(41);

            assert_ok!(LaunchPad::claim_campaign_fundraise(
                Origin::signed(ALICE),
                TEST,
            ));
            
            assert_noop!(
                LaunchPad::claim_campaign_fundraise(
                    Origin::signed(CHARLIE),
                    TEST,
                ),
                Error::<Runtime>::WrongOrigin
            );
        });
}

#[test]
fn claim_campaign_fundraise_does_not_work_already_claimed() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
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
                is_claimed: true,
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            LaunchPad::on_initialize(23);
            assert_ok!(LaunchPad::contribute(
                Origin::signed(BOB),
                TEST,
                10_000
            ));

            assert_ok!(LaunchPad::contribute(
                Origin::signed(ALICE),
                TEST,
                10_000
            ));

            LaunchPad::on_initialize(41);
            System::set_block_number(41);

            assert_noop!(
                LaunchPad::claim_campaign_fundraise(
                    Origin::signed(BOB),
                    TEST,
                ),
                Error::<Runtime>::CampaignAlreadyClaimed
            );
        });
}

#[test]
fn approve_proposal_works() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 21,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            assert_ok!(LaunchPad::contribute(
                Origin::signed(BOB),
                TEST,
                10_000
            ));
        });
}

#[test]
fn approve_proposal_does_not_work() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 21,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: true,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_noop!(
                LaunchPad::approve_proposal(
                    Origin::signed(11),
                    TEST,
                ),
                Error::<Runtime>::ProposalAlreadyApproved
            );
            assert_noop!(
                LaunchPad::approve_proposal(
                    Origin::signed(11),
                    USSD,
                ),
                Error::<Runtime>::ProposalNotFound
            );
        });
}

#[test]
fn reject_proposal_works() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 21,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_ok!(LaunchPad::reject_proposal(
                Origin::signed(11),
                TEST,
            ));
        });
}

#[test]
fn reject_proposal_does_not_work() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 21,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: true,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_noop!(
                LaunchPad::reject_proposal(
                    Origin::signed(11),
                    TEST,
                ),
                Error::<Runtime>::ProposalAlreadyApproved
            );
            assert_noop!(
                LaunchPad::reject_proposal(
                    Origin::signed(11),
                    USSD,
                ),
                Error::<Runtime>::ProposalNotFound
            );
        });
}

#[test]
fn get_contributors_count_works() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            let proposal = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 21,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, proposal.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Proposal should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            assert_ok!(LaunchPad::contribute(
                Origin::signed(ALICE),
                TEST,
                10_000
            ));
            assert_ok!(LaunchPad::contribute(
                Origin::signed(BOB),
                TEST,
                10_000
            ));
            assert_ok!(LaunchPad::contribute(
                Origin::signed(CHARLIE),
                TEST,
                10_000
            ));
            
            assert_eq!(LaunchPad::get_contributors_count(TEST), 3);
        });
}

#[test]
fn get_total_amounts_raised_works() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            assert_eq!(
                LaunchPad::get_total_amounts_raised(),
                vec![]
            );
            let campaign = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 21,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, campaign.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Campaign should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            assert_ok!(LaunchPad::contribute(
                Origin::signed(ALICE),
                TEST,
                50_000
            ));

            LaunchPad::on_initialize(40);

            assert_ok!(LaunchPad::on_successful_campaign(<frame_system::Pallet<Runtime>>::block_number(), TEST));
            assert_eq!(
                LaunchPad::get_total_amounts_raised(),
                vec![
                    (USSD, 50000),
                ]
            );
        });
}

#[test]
fn on_retire_works() {
    ExtBuilder::default()
        .one_hundred_thousand_for_all()
        .build()
        .execute_with(|| {
            assert_eq!(
                LaunchPad::get_total_amounts_raised(),
                vec![]
            );
            let campaign = CampaignInfo {
                id: TEST,
                origin: ALICE.clone(),
                beneficiary: BOB,
                pool: LaunchPad::campaign_pool(0),
                raise_currency: USSD,
                sale_token: TEST,
                token_price: 10,
                crowd_allocation: 10_000,
                goal: 100_000,
                raised: 0,
                contributors_count: 0,
                contributions: Vec::new(),
                period: 20,
                campaign_start: 21,
                campaign_end: 0,
                campaign_retirement_period: 0,
                proposal_retirement_period: 0,
                is_approved: false,
                is_rejected: false,
                is_waiting: true,
                is_active: true,
                is_successful: false,
                is_failed: false,
                is_ended: false,
                is_claimed: false,
            };
            <Proposals<Runtime>>::insert(TEST, campaign.clone());
            assert!(
                <Proposals<Runtime>>::contains_key(TEST),
                "Campaign should be in storage"
            );
            
            assert_ok!(LaunchPad::approve_proposal(
                Origin::signed(11),
                TEST,
            ));
            
            LaunchPad::on_initialize(21);

            assert_ok!(LaunchPad::contribute(
                Origin::signed(ALICE),
                TEST,
                50_000
            ));

            LaunchPad::on_initialize(60);

            assert_ok!(LaunchPad::on_retire(TEST));
        });
}
