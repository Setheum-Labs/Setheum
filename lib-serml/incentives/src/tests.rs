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
		assert_ok!(TokensModule::deposit(CHFJ_SETT_LP, &ALICE, 10000));
		assert_eq!(TokensModule::free_balance(CHFJ_SETT_LP, &ALICE), 10000);
		assert_eq!(
			TokensModule::free_balance(CHFJ_SETT_LP, &IncentivesModule::account_id()),
			0
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexSetterReward(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexIncentive(CHFJ_SETT_LP), ALICE),
			(0, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexSetterReward(CHFJ_SETT_LP), ALICE),
			(0, 0)
		);

		assert_ok!(IncentivesModule::deposit_dex_share(
			Origin::signed(ALICE),
			CHFJ_SETT_LP,
			10000
		));
		let deposit_dex_share_event = Event::incentives(crate::Event::DepositDEXShare(ALICE, CHFJ_SETT_LP, 10000));
		assert!(System::events()
			.iter()
			.any(|record| record.event == deposit_dex_share_event));

		assert_eq!(TokensModule::free_balance(CHFJ_SETT_LP, &ALICE), 0);
		assert_eq!(
			TokensModule::free_balance(CHFJ_SETT_LP, &IncentivesModule::account_id()),
			10000
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexSetterReward(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexIncentive(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexSetterReward(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);
	});
}

#[test]
fn withdraw_dex_share_works() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(TokensModule::deposit(CHFJ_SETT_LP, &ALICE, 10000));

		assert_noop!(
			IncentivesModule::withdraw_dex_share(Origin::signed(BOB), CHFJ_SETT_LP, 10000),
			Error::<Runtime>::NotEnough,
		);

		assert_ok!(IncentivesModule::deposit_dex_share(
			Origin::signed(ALICE),
			CHFJ_SETT_LP,
			10000
		));
		assert_eq!(TokensModule::free_balance(CHFJ_SETT_LP, &ALICE), 0);
		assert_eq!(
			TokensModule::free_balance(CHFJ_SETT_LP, &IncentivesModule::account_id()),
			10000
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexSetterReward(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexIncentive(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexSetterReward(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);

		assert_ok!(IncentivesModule::withdraw_dex_share(
			Origin::signed(ALICE),
			CHFJ_SETT_LP,
			8000
		));
		let withdraw_dex_share_event = Event::incentives(crate::Event::WithdrawDEXShare(ALICE, CHFJ_SETT_LP, 8000));
		assert!(System::events()
			.iter()
			.any(|record| record.event == withdraw_dex_share_event));

		assert_eq!(TokensModule::free_balance(CHFJ_SETT_LP, &ALICE), 8000);
		assert_eq!(
			TokensModule::free_balance(CHFJ_SETT_LP, &IncentivesModule::account_id()),
			2000
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 2000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexSetterReward(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 2000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexIncentive(CHFJ_SETT_LP), ALICE),
			(2000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexSetterReward(CHFJ_SETT_LP), ALICE),
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
			IncentivesModule::incentive_reward_amount(PoolId::DexIncentive(DNAR_SETT_LP)),
			0
		);

		assert_ok!(IncentivesModule::update_incentive_rewards(
			Origin::signed(4),
			vec![
				(PoolId::DexIncentive(DNAR_SETT_LP), 1000)
			],
		));
		assert_eq!(
			IncentivesModule::incentive_reward_amount(PoolId::DexIncentive(DNAR_SETT_LP)),
			1000
		);

		assert_noop!(
			IncentivesModule::update_incentive_rewards(Origin::signed(4), vec![(PoolId::DexIncentive(DNAR), 800)],),
			Error::<Runtime>::InvalidCurrencyId
		);
	});
}

#[test]
fn update_dex_setter_rewards_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			IncentivesModule::update_dex_setter_rewards(Origin::signed(ALICE), vec![]),
			BadOrigin
		);
		assert_noop!(
			IncentivesModule::update_dex_setter_rewards(
				Origin::signed(4),
				vec![(PoolId::DexIncentive(DNAR_SETT_LP), Rate::zero())]
			),
			Error::<Runtime>::InvalidPoolId
		);
		assert_noop!(
			IncentivesModule::update_dex_setter_rewards(
				Origin::signed(4),
				vec![(PoolId::DexSetterReward(DNAR), Rate::zero())]
			),
			Error::<Runtime>::InvalidCurrencyId
		);

		assert_eq!(
			IncentivesModule::dex_setter_reward_rate(PoolId::DexSetterReward(DNAR_SETT_LP)),
			Rate::zero()
		);
		assert_ok!(IncentivesModule::update_dex_setter_rewards(
			Origin::signed(4),
			vec![(PoolId::DexSetterReward(DNAR_SETT_LP), Rate::saturating_from_rational(1, 100)),]
		));
		assert_eq!(
			IncentivesModule::dex_setter_reward_rate(PoolId::DexSetterReward(DNAR_SETT_LP)),
			Rate::saturating_from_rational(1, 100)
		);
	});
}

#[test]
fn pay_out_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(TokensModule::deposit(SDEX, &VAULT, 10000));
		assert_ok!(TokensModule::deposit(SETT, &VAULT, 10000));

		assert_eq!(TokensModule::free_balance(SDEX, &VAULT), 10000);
		assert_eq!(TokensModule::free_balance(SDEX, &BOB), 0);
		IncentivesModule::payout(&BOB, PoolId::DexIncentive(DNAR_SETT_LP), 1000);
		assert_eq!(TokensModule::free_balance(SDEX, &VAULT), 9000);
		assert_eq!(TokensModule::free_balance(SDEX, &BOB), 1000);

		assert_eq!(TokensModule::free_balance(SETT, &VAULT), 10000);
		assert_eq!(TokensModule::free_balance(SETT, &ALICE), 0);
		IncentivesModule::payout(&ALICE, PoolId::DexSetterReward(DBAR_SETT_LP), 1000);
		assert_eq!(TokensModule::free_balance(SETT, &VAULT), 9000);
		assert_eq!(TokensModule::free_balance(SETT, &ALICE), 1000);
	});
}

#[test]
fn on_initialize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(IncentivesModule::update_incentive_rewards(
			Origin::signed(4),
			vec![
				(PoolId::DexIncentive(CHFJ_SETT_LP), 100),
				(PoolId::DexIncentive(DNAR_SETT_LP), 200),
			],
		));
		assert_ok!(IncentivesModule::update_dex_setter_rewards(
			Origin::signed(4),
			vec![
				(PoolId::DexSetterReward(CHFJ_SETT_LP), Rate::saturating_from_rational(1, 100)),
				(PoolId::DexSetterReward(DNAR_SETT_LP), Rate::saturating_from_rational(1, 100)),
			],
		));

		RewardsModule::add_share(&ALICE, &PoolId::DexIncentive(CHFJ_SETT_LP), 1);
		RewardsModule::add_share(&ALICE, &PoolId::DexIncentive(DNAR_SETT_LP), 1);
		RewardsModule::add_share(&ALICE, &PoolId::DexSetterReward(CHFJ_SETT_LP), 1);
		RewardsModule::add_share(&ALICE, &PoolId::DexSetterReward(DNAR_SETT_LP), 1);
		assert_eq!(TokensModule::free_balance(SDEX, &VAULT), 0);
		assert_eq!(TokensModule::free_balance(SETT, &VAULT), 0);
		assert_eq!(RewardsModule::pools(PoolId::DexIncentive(CHFJ_SETT_LP)).total_rewards, 0);
		assert_eq!(RewardsModule::pools(PoolId::DexIncentive(DNAR_SETT_LP)).total_rewards, 0);
		assert_eq!(RewardsModule::pools(PoolId::DexSetterReward(CHFJ_SETT_LP)).total_rewards, 0);
		assert_eq!(RewardsModule::pools(PoolId::DexSetterReward(DNAR_SETT_LP)).total_rewards, 0);

		IncentivesModule::on_initialize(9);
		assert_eq!(TokensModule::free_balance(SDEX, &VAULT), 0);
		assert_eq!(TokensModule::free_balance(SETT, &VAULT), 0);
		assert_eq!(RewardsModule::pools(PoolId::DexIncentive(CHFJ_SETT_LP)).total_rewards, 0);
		assert_eq!(RewardsModule::pools(PoolId::DexIncentive(DNAR_SETT_LP)).total_rewards, 0);
		assert_eq!(RewardsModule::pools(PoolId::DexSetterReward(CHFJ_SETT_LP)).total_rewards, 0);
		assert_eq!(RewardsModule::pools(PoolId::DexSetterReward(DNAR_SETT_LP)).total_rewards, 0);

		IncentivesModule::on_initialize(10);
		assert_eq!(TokensModule::free_balance(SDEX, &VAULT), 1300);
		assert_eq!(TokensModule::free_balance(SETT, &VAULT), 9);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(CHFJ_SETT_LP)).total_rewards,
			1100
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(DNAR_SETT_LP)).total_rewards,
			1500
		);
		assert_eq!(RewardsModule::pools(PoolId::DexSetterReward(CHFJ_SETT_LP)).total_rewards, 5);
		assert_eq!(RewardsModule::pools(PoolId::DexSetterReward(DNAR_SETT_LP)).total_rewards, 4);

		IncentivesModule::on_initialize(20);
		assert_eq!(TokensModule::free_balance(SDEX, &VAULT), 4630);
		assert_eq!(TokensModule::free_balance(SETT, &VAULT), 18);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(CHFJ_SETT_LP)).total_rewards,
			2200
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(DNAR_SETT_LP)).total_rewards,
			2400
		);
		assert_eq!(RewardsModule::pools(PoolId::DexSetterReward(CHFJ_SETT_LP)).total_rewards, 10);
		assert_eq!(RewardsModule::pools(PoolId::DexSetterReward(DNAR_SETT_LP)).total_rewards, 38);

		mock_shutdown();
		IncentivesModule::on_initialize(30);
		assert_eq!(TokensModule::free_balance(SDEX, &VAULT), 4630);
		assert_eq!(TokensModule::free_balance(SETT, &VAULT), 18);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(CHFJ_SETT_LP)).total_rewards,
			2200
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexIncentive(DNAR_SETT_LP)).total_rewards,
			2400
		);
		assert_eq!(RewardsModule::pools(PoolId::DexSetterReward(CHFJ_SETT_LP)).total_rewards, 10);
		assert_eq!(RewardsModule::pools(PoolId::DexSetterReward(DNAR_SETT_LP)).total_rewards, 38);
	});
}
