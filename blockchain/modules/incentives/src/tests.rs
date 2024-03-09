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

//! Unit tests for the incentives module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{RuntimeEvent, *};
use orml_rewards::PoolInfo;
use orml_traits::MultiCurrency;
use sp_runtime::{traits::BadOrigin, FixedPointNumber};

#[test]
fn deposit_dex_share_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(TokensModule::deposit(BTC_USSD_LP, &ALICE::get(), 10000));
		assert_eq!(TokensModule::free_balance(BTC_USSD_LP, &ALICE::get()), 10000);
		assert_eq!(
			TokensModule::free_balance(BTC_USSD_LP, &IncentivesModule::account_id()),
			0
		);
		assert_eq!(RewardsModule::pool_infos(PoolId::Dex(BTC_USSD_LP)), PoolInfo::default(),);

		assert_eq!(
			RewardsModule::shares_and_withdrawn_rewards(PoolId::Dex(BTC_USSD_LP), ALICE::get()),
			Default::default(),
		);

		assert_ok!(IncentivesModule::deposit_dex_share(
			RuntimeOrigin::signed(ALICE::get()),
			BTC_USSD_LP,
			10000
		));
		System::assert_last_event(RuntimeEvent::IncentivesModule(crate::Event::DepositDexShare {
			who: ALICE::get(),
			dex_share_type: BTC_USSD_LP,
			deposit: 10000,
		}));
		assert_eq!(TokensModule::free_balance(BTC_USSD_LP, &ALICE::get()), 0);
		assert_eq!(
			TokensModule::free_balance(BTC_USSD_LP, &IncentivesModule::account_id()),
			10000
		);
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(BTC_USSD_LP)),
			PoolInfo {
				total_shares: 10000,
				..Default::default()
			}
		);
		assert_eq!(
			RewardsModule::shares_and_withdrawn_rewards(PoolId::Dex(BTC_USSD_LP), ALICE::get()),
			(10000, Default::default())
		);
	});
}

#[test]
fn withdraw_dex_share_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(TokensModule::deposit(BTC_USSD_LP, &ALICE::get(), 10000));

		assert_noop!(
			IncentivesModule::withdraw_dex_share(RuntimeOrigin::signed(BOB::get()), BTC_USSD_LP, 10000),
			Error::<Runtime>::NotEnough,
		);

		assert_ok!(IncentivesModule::deposit_dex_share(
			RuntimeOrigin::signed(ALICE::get()),
			BTC_USSD_LP,
			10000
		));
		assert_eq!(TokensModule::free_balance(BTC_USSD_LP, &ALICE::get()), 0);
		assert_eq!(
			TokensModule::free_balance(BTC_USSD_LP, &IncentivesModule::account_id()),
			10000
		);
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(BTC_USSD_LP)),
			PoolInfo {
				total_shares: 10000,
				..Default::default()
			}
		);
		assert_eq!(
			RewardsModule::shares_and_withdrawn_rewards(PoolId::Dex(BTC_USSD_LP), ALICE::get()),
			(10000, Default::default())
		);

		assert_ok!(IncentivesModule::withdraw_dex_share(
			RuntimeOrigin::signed(ALICE::get()),
			BTC_USSD_LP,
			8000
		));
		System::assert_last_event(RuntimeEvent::IncentivesModule(crate::Event::WithdrawDexShare {
			who: ALICE::get(),
			dex_share_type: BTC_USSD_LP,
			withdraw: 8000,
		}));
		assert_eq!(TokensModule::free_balance(BTC_USSD_LP, &ALICE::get()), 8000);
		assert_eq!(
			TokensModule::free_balance(BTC_USSD_LP, &IncentivesModule::account_id()),
			2000
		);
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(BTC_USSD_LP)),
			PoolInfo {
				total_shares: 2000,
				..Default::default()
			}
		);
		assert_eq!(
			RewardsModule::shares_and_withdrawn_rewards(PoolId::Dex(BTC_USSD_LP), ALICE::get()),
			(2000, Default::default())
		);
	});
}

#[test]
fn update_incentive_rewards_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			IncentivesModule::update_incentive_rewards(RuntimeOrigin::signed(ALICE::get()), vec![]),
			BadOrigin
		);
		assert_noop!(
			IncentivesModule::update_incentive_rewards(
				RuntimeOrigin::signed(ROOT::get()),
				vec![(PoolId::Dex(EDF), vec![])]
			),
			Error::<Runtime>::InvalidPoolId
		);

		assert_eq!(
			IncentivesModule::incentive_reward_amounts(PoolId::Dex(EDF_USSD_LP), SEE),
			0
		);
		assert_eq!(
			IncentivesModule::incentive_reward_amounts(PoolId::Dex(EDF_USSD_LP), EDF),
			0
		);
		
		assert_ok!(IncentivesModule::update_incentive_rewards(
			RuntimeOrigin::signed(ROOT::get()),
			vec![
				(PoolId::Dex(EDF_USSD_LP), vec![(SEE, 1000), (EDF, 100)]),
			],
		));
		System::assert_has_event(RuntimeEvent::IncentivesModule(
			crate::Event::IncentiveRewardAmountUpdated {
				pool: PoolId::Dex(EDF_USSD_LP),
				reward_currency_id: SEE,
				reward_amount_per_period: 1000,
			},
		));
		System::assert_has_event(RuntimeEvent::IncentivesModule(
			crate::Event::IncentiveRewardAmountUpdated {
				pool: PoolId::Dex(EDF_USSD_LP),
				reward_currency_id: EDF,
				reward_amount_per_period: 100,
			},
		));
		assert_eq!(
			IncentivesModule::incentive_reward_amounts(PoolId::Dex(EDF_USSD_LP), SEE),
			1000
		);
		assert_eq!(
			IncentiveRewardAmounts::<Runtime>::contains_key(PoolId::Dex(EDF_USSD_LP), EDF),
			true
		);
		assert_eq!(
			IncentivesModule::incentive_reward_amounts(PoolId::Dex(EDF_USSD_LP), EDF),
			100
		);
		
		assert_ok!(IncentivesModule::update_incentive_rewards(
			RuntimeOrigin::signed(ROOT::get()),
			vec![(PoolId::Dex(EDF_USSD_LP), vec![(SEE, 200), (EDF, 0)])],
		));
		System::assert_has_event(RuntimeEvent::IncentivesModule(
			crate::Event::IncentiveRewardAmountUpdated {
				pool: PoolId::Dex(EDF_USSD_LP),
				reward_currency_id: SEE,
				reward_amount_per_period: 200,
			},
		));
		System::assert_has_event(RuntimeEvent::IncentivesModule(
			crate::Event::IncentiveRewardAmountUpdated {
				pool: PoolId::Dex(EDF_USSD_LP),
				reward_currency_id: EDF,
				reward_amount_per_period: 0,
			},
		));
		assert_eq!(
			IncentivesModule::incentive_reward_amounts(PoolId::Dex(EDF_USSD_LP), SEE),
			200
		);
		assert_eq!(
			IncentiveRewardAmounts::<Runtime>::contains_key(PoolId::Dex(EDF_USSD_LP), EDF),
			false
		);
	});
}

#[test]
fn update_claim_reward_deduction_rates_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			IncentivesModule::update_claim_reward_deduction_rates(RuntimeOrigin::signed(ALICE::get()), vec![]),
			BadOrigin
		);
		assert_noop!(
			IncentivesModule::update_claim_reward_deduction_rates(
				RuntimeOrigin::signed(ROOT::get()),
				vec![(PoolId::Dex(EDF), Rate::zero())]
			),
			Error::<Runtime>::InvalidPoolId
		);
		assert_noop!(
			IncentivesModule::update_claim_reward_deduction_rates(
				RuntimeOrigin::signed(ROOT::get()),
				vec![(PoolId::Dex(EDF_USSD_LP), Rate::saturating_from_rational(101, 100)),]
			),
			Error::<Runtime>::InvalidRate,
		);

		assert_eq!(
			IncentivesModule::claim_reward_deduction_rates(&PoolId::Dex(EDF_USSD_LP)),
			Rate::zero()
		);
		assert_eq!(
			IncentivesModule::claim_reward_deduction_rates(&PoolId::Dex(BTC_USSD_LP)),
			Rate::zero()
		);

		assert_ok!(IncentivesModule::update_claim_reward_deduction_rates(
			RuntimeOrigin::signed(ROOT::get()),
			vec![
				(PoolId::Dex(EDF_USSD_LP), Rate::saturating_from_rational(1, 100)),
				(PoolId::Dex(BTC_USSD_LP), Rate::saturating_from_rational(2, 100))
			]
		));
		System::assert_has_event(RuntimeEvent::IncentivesModule(
			crate::Event::ClaimRewardDeductionRateUpdated {
				pool: PoolId::Dex(EDF_USSD_LP),
				deduction_rate: Rate::saturating_from_rational(1, 100),
			},
		));
		System::assert_has_event(RuntimeEvent::IncentivesModule(
			crate::Event::ClaimRewardDeductionRateUpdated {
				pool: PoolId::Dex(BTC_USSD_LP),
				deduction_rate: Rate::saturating_from_rational(2, 100),
			},
		));
		assert_eq!(
			IncentivesModule::claim_reward_deduction_rates(&PoolId::Dex(EDF_USSD_LP)),
			Rate::saturating_from_rational(1, 100)
		);
		assert_eq!(
			ClaimRewardDeductionRates::<Runtime>::contains_key(PoolId::Dex(BTC_USSD_LP)),
			true
		);
		assert_eq!(
			IncentivesModule::claim_reward_deduction_rates(&PoolId::Dex(BTC_USSD_LP)),
			Rate::saturating_from_rational(2, 100)
		);

		assert_ok!(IncentivesModule::update_claim_reward_deduction_rates(
			RuntimeOrigin::signed(ROOT::get()),
			vec![
				(PoolId::Dex(EDF_USSD_LP), Rate::saturating_from_rational(5, 100)),
				(PoolId::Dex(BTC_USSD_LP), Rate::zero())
			]
		));
		System::assert_has_event(RuntimeEvent::IncentivesModule(
			crate::Event::ClaimRewardDeductionRateUpdated {
				pool: PoolId::Dex(EDF_USSD_LP),
				deduction_rate: Rate::saturating_from_rational(5, 100),
			},
		));
		System::assert_has_event(RuntimeEvent::IncentivesModule(
			crate::Event::ClaimRewardDeductionRateUpdated {
				pool: PoolId::Dex(BTC_USSD_LP),
				deduction_rate: Rate::zero(),
			},
		));
		assert_eq!(
			IncentivesModule::claim_reward_deduction_rates(&PoolId::Dex(EDF_USSD_LP)),
			Rate::saturating_from_rational(5, 100)
		);
		assert_eq!(
			ClaimRewardDeductionRates::<Runtime>::contains_key(PoolId::Dex(BTC_USSD_LP)),
			false
		);
		assert_eq!(
			IncentivesModule::claim_reward_deduction_rates(&PoolId::Dex(BTC_USSD_LP)),
			Rate::zero()
		);
	});
}

#[test]
fn payout_works() {
	ExtBuilder::default().build().execute_with(|| {
		
	});
}

#[test]
fn transfer_failed_when_claim_rewards() {
	ExtBuilder::default().build().execute_with(|| {

	});
}

#[test]
fn claim_rewards_works() {
	ExtBuilder::default().build().execute_with(|| {
		
	});
}

#[test]
fn on_initialize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(TokensModule::deposit(SEE, &RewardsSource::get(), 10000));
		assert_ok!(TokensModule::deposit(USSD, &RewardsSource::get(), 10000));
		assert_ok!(TokensModule::deposit(LEDF, &RewardsSource::get(), 10000));

		assert_ok!(IncentivesModule::update_incentive_rewards(
			RuntimeOrigin::signed(ROOT::get()),
			vec![
				(PoolId::Dex(BTC_USSD_LP), vec![(SEE, 100)]),
				(PoolId::Dex(EDF_USSD_LP), vec![(SEE, 200)]),
				(PoolId::MoyaEarnRewards(SEE), vec![(SEE, 100)]),
			],
		));

		RewardsModule::add_share(&ALICE::get(), &PoolId::Dex(BTC_USSD_LP), 1);
		RewardsModule::add_share(&ALICE::get(), &PoolId::Dex(EDF_USSD_LP), 1);
		RewardsModule::add_share(&ALICE::get(), &PoolId::MoyaEarnRewards(SEE), 1);

		assert_eq!(TokensModule::free_balance(SEE, &RewardsSource::get()), 10000);
		assert_eq!(TokensModule::free_balance(USSD, &RewardsSource::get()), 10000);
		assert_eq!(TokensModule::free_balance(LEDF, &RewardsSource::get()), 10000);
		
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(BTC_USSD_LP)),
			PoolInfo {
				total_shares: 1,
				..Default::default()
			}
		);
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(EDF_USSD_LP)),
			PoolInfo {
				total_shares: 1,
				..Default::default()
			}
		);
		assert_eq!(
			RewardsModule::pool_infos(PoolId::MoyaEarnRewards(SEE)),
			PoolInfo {
				total_shares: 1,
				..Default::default()
			}
		);

		// per 10 blocks will accumulate rewards, nothing happened when on_initialize(9)
		IncentivesModule::on_initialize(9);

		IncentivesModule::on_initialize(10);
		assert_eq!(
			TokensModule::free_balance(SEE, &RewardsSource::get()),
			10000 - (1000 + 200 + 100 + 100)
		);
		assert_eq!(TokensModule::free_balance(USSD, &RewardsSource::get()), 10000 - 500);
		assert_eq!(TokensModule::free_balance(LEDF, &RewardsSource::get()), 10000);
		
		// 100 SEE is incentive reward
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(BTC_USSD_LP)),
			PoolInfo {
				total_shares: 1,
				rewards: vec![(SEE, (100, 0))].into_iter().collect(),
			}
		);
		// 200 SEE is incentive reward
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(EDF_USSD_LP)),
			PoolInfo {
				total_shares: 1,
				rewards: vec![(SEE, (200, 0))].into_iter().collect(),
			}
		);
		// 100 SEE is incentive reward
		assert_eq!(
			RewardsModule::pool_infos(PoolId::MoyaEarnRewards(SEE)),
			PoolInfo {
				total_shares: 1,
				rewards: vec![(SEE, (100, 0))].into_iter().collect(),
			}
		);

		IncentivesModule::on_initialize(20);
		assert_eq!(
			TokensModule::free_balance(SEE, &RewardsSource::get()),
			8600 - (1000 + 2000 + 100 + 200 + 100)
		);
		assert_eq!(TokensModule::free_balance(USSD, &RewardsSource::get()), 9500 - 500);
		assert_eq!(TokensModule::free_balance(LEDF, &RewardsSource::get()), 10000 - 50);
		
		// 100 SEE is incentive reward
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(BTC_USSD_LP)),
			PoolInfo {
				total_shares: 1,
				rewards: vec![(SEE, (200, 0))].into_iter().collect(),
			}
		);
		// 200 SEE is incentive reward
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(EDF_USSD_LP)),
			PoolInfo {
				total_shares: 1,
				rewards: vec![(SEE, (400, 0))].into_iter().collect(),
			}
		);
		// 100 SEE is incentive reward
		assert_eq!(
			RewardsModule::pool_infos(PoolId::MoyaEarnRewards(SEE)),
			PoolInfo {
				total_shares: 1,
				rewards: vec![(SEE, (200, 0))].into_iter().collect(),
			}
		);

		mock_shutdown();
		IncentivesModule::on_initialize(30);
		assert_eq!(
			TokensModule::free_balance(SEE, &RewardsSource::get()),
			5200 - (100 + 200 + 100)
		);
		assert_eq!(TokensModule::free_balance(USSD, &RewardsSource::get()), 9000);
		assert_eq!(TokensModule::free_balance(LEDF, &RewardsSource::get()), 9950);
		
		// after shutdown, PoolId::Dex will accumulate incentive rewards
		// reward
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(BTC_USSD_LP)),
			PoolInfo {
				total_shares: 1,
				rewards: vec![(SEE, (300, 0))].into_iter().collect(),
			}
		);
		// after shutdown, PoolId::Dex will accumulate incentive rewards
		// reward
		assert_eq!(
			RewardsModule::pool_infos(PoolId::Dex(EDF_USSD_LP)),
			PoolInfo {
				total_shares: 1,
				rewards: vec![(SEE, (600, 0))].into_iter().collect(),
			}
		);
		// after shutdown, PoolId::MoyaEarnRewards will accumulate incentive rewards
		// reward
		assert_eq!(
			RewardsModule::pool_infos(PoolId::MoyaEarnRewards(SEE)),
			PoolInfo {
				total_shares: 1,
				rewards: vec![(SEE, (300, 0))].into_iter().collect(),
			}
		);
	});
}

#[test]
fn earning_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		OnEarnBonded::<Runtime>::happened(&(ALICE::get(), 80));
		assert_eq!(
			RewardsModule::pool_infos(PoolId::MoyaEarnRewards(SEE)),
			PoolInfo {
				total_shares: 80,
				..Default::default()
			}
		);
		assert_eq!(
			RewardsModule::shares_and_withdrawn_rewards(PoolId::MoyaEarnRewards(SEE), ALICE::get()),
			(80, Default::default())
		);

		OnEarnUnbonded::<Runtime>::happened(&(ALICE::get(), 20));
		assert_eq!(
			RewardsModule::pool_infos(PoolId::MoyaEarnRewards(SEE)),
			PoolInfo {
				total_shares: 60,
				..Default::default()
			}
		);
		assert_eq!(
			RewardsModule::shares_and_withdrawn_rewards(PoolId::MoyaEarnRewards(SEE), ALICE::get()),
			(60, Default::default())
		);

		OnEarnUnbonded::<Runtime>::happened(&(ALICE::get(), 60));
		assert_eq!(
			RewardsModule::pool_infos(PoolId::MoyaEarnRewards(SEE)),
			PoolInfo { ..Default::default() }
		);
		assert_eq!(
			RewardsModule::shares_and_withdrawn_rewards(PoolId::MoyaEarnRewards(SEE), ALICE::get()),
			(0, Default::default())
		);
	});
}

#[test]
fn transfer_reward_and_update_rewards_storage_atomically_when_accumulate_incentives_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(TokensModule::deposit(USSD, &RewardsSource::get(), 100));
		assert_ok!(TokensModule::deposit(SEE, &RewardsSource::get(), 100));
		assert_eq!(TokensModule::free_balance(SEE, &RewardsSource::get()), 100);
		assert_eq!(TokensModule::free_balance(USSD, &RewardsSource::get()), 100);
		assert_eq!(
			orml_rewards::PoolInfos::<Runtime>::contains_key(PoolId::Dex(LEDF)),
			false
		);

		assert_eq!(TokensModule::free_balance(SEE, &RewardsSource::get()), 100);
		assert_eq!(TokensModule::free_balance(USSD, &RewardsSource::get()), 100);
		
		// accumulate SEE and USSD rewards succeeded
		assert_eq!(TokensModule::free_balance(SEE, &RewardsSource::get()), 70);
		assert_eq!(TokensModule::free_balance(USSD, &RewardsSource::get()), 10);
		
		// accumulate SEE reward succeeded， accumulate USSD reward failed
		assert_eq!(TokensModule::free_balance(SEE, &RewardsSource::get()), 40);
		assert_eq!(TokensModule::free_balance(USSD, &RewardsSource::get()), 10);
	});
}

#[test]
fn update_claim_reward_deduction_currency() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			IncentivesModule::update_claim_reward_deduction_currency(
				RuntimeOrigin::signed(ALICE::get()),
				PoolId::Dex(EDF_USSD_LP),
				Some(SEE)
			),
			BadOrigin
		);

		assert_ok!(IncentivesModule::update_claim_reward_deduction_rates(
			RuntimeOrigin::signed(ROOT::get()),
			vec![(PoolId::Dex(EDF_USSD_LP), Rate::saturating_from_rational(10, 100)),]
		));
		assert_ok!(IncentivesModule::update_claim_reward_deduction_currency(
			RuntimeOrigin::signed(ROOT::get()),
			PoolId::Dex(EDF_USSD_LP),
			Some(SEE)
		),);
		System::assert_has_event(RuntimeEvent::IncentivesModule(
			crate::Event::ClaimRewardDeductionCurrencyUpdated {
				pool: PoolId::Dex(EDF_USSD_LP),
				currency: Some(SEE),
			},
		));

		assert_eq!(
			ClaimRewardDeductionCurrency::<Runtime>::get(PoolId::Dex(EDF_USSD_LP)),
			Some(SEE)
		);
	});
}

#[test]
fn claim_reward_deduction_currency_works() {
	ExtBuilder::default().build().execute_with(|| {
		let pool_id = PoolId::Dex(EDF_USSD_LP);

		assert_ok!(IncentivesModule::update_claim_reward_deduction_rates(
			RuntimeOrigin::signed(ROOT::get()),
			vec![(pool_id, Rate::saturating_from_rational(10, 100)),]
		));
		assert_ok!(IncentivesModule::update_claim_reward_deduction_currency(
			RuntimeOrigin::signed(ROOT::get()),
			pool_id,
			Some(SEE)
		));

		// alice add shares before accumulate rewards
		RewardsModule::add_share(&ALICE::get(), &pool_id, 100);

		// bob add shares before accumulate rewards
		RewardsModule::add_share(&BOB::get(), &pool_id, 100);

		// accumulate rewards
		assert_ok!(RewardsModule::accumulate_reward(&pool_id, SEE, 1000));
		assert_ok!(RewardsModule::accumulate_reward(&pool_id, USSD, 2000));

		// alice claim rewards
		assert_ok!(IncentivesModule::claim_rewards(
			RuntimeOrigin::signed(ALICE::get()),
			pool_id
		));

		System::assert_has_event(RuntimeEvent::IncentivesModule(crate::Event::ClaimRewards {
			who: ALICE::get(),
			pool: pool_id,
			reward_currency_id: SEE,
			actual_amount: 450,
			deduction_amount: 50,
		}));
		System::assert_has_event(RuntimeEvent::IncentivesModule(crate::Event::ClaimRewards {
			who: ALICE::get(),
			pool: pool_id,
			reward_currency_id: USSD,
			actual_amount: 1000,
			deduction_amount: 0,
		}));

		System::reset_events();

		assert_eq!(TokensModule::free_balance(SEE, &ALICE::get()), 450);
		assert_eq!(TokensModule::free_balance(USSD, &ALICE::get()), 1000);

		// apply deduction currency to all rewards
		assert_ok!(IncentivesModule::update_claim_reward_deduction_currency(
			RuntimeOrigin::signed(ROOT::get()),
			pool_id,
			None
		));

		// accumulate rewards
		assert_ok!(RewardsModule::accumulate_reward(&pool_id, SEE, 1000));
		assert_ok!(RewardsModule::accumulate_reward(&pool_id, USSD, 2000));

		// alice claim rewards
		assert_ok!(IncentivesModule::claim_rewards(
			RuntimeOrigin::signed(ALICE::get()),
			pool_id
		));

		System::assert_has_event(RuntimeEvent::IncentivesModule(crate::Event::ClaimRewards {
			who: ALICE::get(),
			pool: pool_id,
			reward_currency_id: SEE,
			actual_amount: 473,
			deduction_amount: 52,
		}));
		System::assert_has_event(RuntimeEvent::IncentivesModule(crate::Event::ClaimRewards {
			who: ALICE::get(),
			pool: pool_id,
			reward_currency_id: USSD,
			actual_amount: 900,
			deduction_amount: 100,
		}));

		assert_eq!(TokensModule::free_balance(SEE, &ALICE::get()), 923);
		assert_eq!(TokensModule::free_balance(USSD, &ALICE::get()), 1900);
	});
}
