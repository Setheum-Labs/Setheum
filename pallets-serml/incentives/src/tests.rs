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

//! Unit tests for the incentives module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use orml_rewards::PoolInfo;
use orml_traits::MultiCurrency;
use sp_runtime::{traits::BadOrigin, FixedPointNumber};

#[test]
fn deposit_dex_share_works() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(TokensModule::deposit(BTC_JUSD_LP, &ALICE, 10000));
		assert_eq!(TokensModule::free_balance(BTC_JUSD_LP, &ALICE), 10000);
		assert_eq!(
			TokensModule::free_balance(BTC_JUSD_LP, &IncentivesModule::account_id()),
			0
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(BTC_JUSD_LP)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexSaving(BTC_JUSD_LP)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexIncentive(BTC_JUSD_LP), ALICE),
			(0, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexSaving(BTC_JUSD_LP), ALICE),
			(0, 0)
		);

		assert_ok!(IncentivesModule::deposit_dex_share(
			Origin::signed(ALICE),
			BTC_JUSD_LP,
			10000
		));
		let deposit_dex_share_event = Event::incentives(crate::Event::DepositDEXShare(ALICE, BTC_JUSD_LP, 10000));
		assert!(System::events()
			.iter()
			.any(|record| record.event == deposit_dex_share_event));

		assert_eq!(TokensModule::free_balance(BTC_JUSD_LP, &ALICE), 0);
		assert_eq!(
			TokensModule::free_balance(BTC_JUSD_LP, &IncentivesModule::account_id()),
			10000
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(BTC_JUSD_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexSaving(BTC_JUSD_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexIncentive(BTC_JUSD_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexSaving(BTC_JUSD_LP), ALICE),
			(10000, 0)
		);
	});
}

#[test]
fn withdraw_dex_share_works() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(TokensModule::deposit(BTC_JUSD_LP, &ALICE, 10000));

		assert_noop!(
			IncentivesModule::withdraw_dex_share(Origin::signed(BOB), BTC_JUSD_LP, 10000),
			Error::<Runtime>::NotEnough,
		);

		assert_ok!(IncentivesModule::deposit_dex_share(
			Origin::signed(ALICE),
			BTC_JUSD_LP,
			10000
		));
		assert_eq!(TokensModule::free_balance(BTC_JUSD_LP, &ALICE), 0);
		assert_eq!(
			TokensModule::free_balance(BTC_JUSD_LP, &IncentivesModule::account_id()),
			10000
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(BTC_JUSD_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexSaving(BTC_JUSD_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexIncentive(BTC_JUSD_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexSaving(BTC_JUSD_LP), ALICE),
			(10000, 0)
		);

		assert_ok!(IncentivesModule::withdraw_dex_share(
			Origin::signed(ALICE),
			BTC_JUSD_LP,
			8000
		));
		let withdraw_dex_share_event = Event::incentives(crate::Event::WithdrawDEXShare(ALICE, BTC_JUSD_LP, 8000));
		assert!(System::events()
			.iter()
			.any(|record| record.event == withdraw_dex_share_event));

		assert_eq!(TokensModule::free_balance(BTC_JUSD_LP, &ALICE), 8000);
		assert_eq!(
			TokensModule::free_balance(BTC_JUSD_LP, &IncentivesModule::account_id()),
			2000
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(BTC_JUSD_LP)),
			PoolInfo {
				total_shares: 2000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexSaving(BTC_JUSD_LP)),
			PoolInfo {
				total_shares: 2000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexIncentive(BTC_JUSD_LP), ALICE),
			(2000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexSaving(BTC_JUSD_LP), ALICE),
			(2000, 0)
		);
	});
}

#[test]
fn update_incentive_rewards_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			IncentivesModule::update_incentive_rewards(Origin::signed(ALICE), vec![]),
			BadOrigin
		);

		assert_eq!(
			IncentivesModule::incentive_reward_amount(PoolId::DexIncentive(DOT_JUSD_LP)),
			0
		);

		assert_ok!(IncentivesModule::update_incentive_rewards(
			Origin::signed(4),
			vec![
				(PoolId::DexIncentive(DOT_JUSD_LP), 1000),
		));
		assert_eq!(
			IncentivesModule::incentive_reward_amount(PoolId::DexIncentive(DOT_JUSD_LP)),
			1000
		);
		assert_noop!(
			IncentivesModule::update_incentive_rewards(Origin::signed(4), vec![(PoolId::DexIncentive(DOT), 800)],),
			Error::<Runtime>::InvalidCurrencyId
		);
	});
}

#[test]
fn pay_out_works_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(TokensModule::deposit(DNAR, &VAULT, 10000));
		assert_ok!(TokensModule::deposit(JUSD, &VAULT, 10000));

		assert_eq!(TokensModule::free_balance(DNAR, &BOB), 0);
		IncentivesModule::payout(&BOB, &PoolId::DexIncentive(DOT_JUSD_LP), 1000);
		assert_eq!(TokensModule::free_balance(DNAR, &VAULT), 8000);
		assert_eq!(TokensModule::free_balance(DNAR, &BOB), 1000);
	});
}


//TODO: Re-implement on_initialize and repair it.__rust_force_expr!

#[test]
fn on_initialize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(IncentivesModule::update_incentive_rewards(
			Origin::signed(4),
			vec![
				(PoolId::DexIncentive(BTC_JUSD_LP), 100),
				(PoolId::DexIncentive(DOT_JUSD_LP), 200),
			],
		));

		RewardsModule::add_share(&ALICE, &PoolId::DexIncentive(BTC_JUSD_LP), 1);
		RewardsModule::add_share(&ALICE, &PoolId::DexIncentive(DOT_JUSD_LP), 1);
		assert_eq!(TokensModule::free_balance(DNAR, &VAULT), 0);
		assert_eq!(TokensModule::free_balance(JUSD, &VAULT), 0);
		assert_eq!(RewardsModule::pools(PoolId::DexIncentive(BTC_JUSD_LP)).total_rewards, 0);
		assert_eq!(RewardsModule::pools(PoolId::DexIncentive(DOT_JUSD_LP)).total_rewards, 0);

		IncentivesModule::on_initialize(9);
		assert_eq!(TokensModule::free_balance(DNAR, &VAULT), 0);
		assert_eq!(TokensModule::free_balance(JUSD, &VAULT), 0);
		assert_eq!(RewardsModule::pools(PoolId::DexIncentive(BTC_JUSD_LP)).total_rewards, 0);
		assert_eq!(RewardsModule::pools(PoolId::DexIncentive(DOT_JUSD_LP)).total_rewards, 0);

		IncentivesModule::on_initialize(10);
		assert_eq!(TokensModule::free_balance(DNAR, &VAULT), 1300);
		assert_eq!(TokensModule::free_balance(JUSD, &VAULT), 9);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(BTC_JUSD_LP)).total_rewards,
			100
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(DOT_JUSD_LP)).total_rewards,
			200
		);
		IncentivesModule::on_initialize(20);
		assert_eq!(TokensModule::free_balance(DNAR, &VAULT), 4630);
		assert_eq!(TokensModule::free_balance(JUSD, &VAULT), 18);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(BTC_JUSD_LP)).total_rewards,
			200
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(DOT_JUSD_LP)).total_rewards,
			400
		);
	});
}
