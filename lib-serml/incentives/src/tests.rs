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
			RewardsModule::pools(PoolId::DexPremium(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexPlus(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexBonus(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexExtra(CHFJ_SETT_LP)),
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
			RewardsModule::share_and_withdrawn_reward(PoolId::DexPremium(CHFJ_SETT_LP), ALICE),
			(0, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexPlus(CHFJ_SETT_LP), ALICE),
			(0, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexBonus(CHFJ_SETT_LP), ALICE),
			(0, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexExtra(CHFJ_SETT_LP), ALICE),
			(0, 0)
		);

		assert_ok!(IncentivesModule::deposit_dex_share(
			Origin::signed(ALICE),
			CHFJ_SETT_LP,
			10000
		));
		System::assert_last_event(Event::incentives(crate::Event::DepositDEXShare(ALICE, CHFJ_SETT_LP, 10000)));

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
			RewardsModule::pools(PoolId::DexPremium(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexPlus(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexBonus(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexExtra(CHFJ_SETT_LP)),
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
			RewardsModule::share_and_withdrawn_reward(PoolId::DexPremium(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexPlus(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexBonus(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexExtra(CHFJ_SETT_LP), ALICE),
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
			RewardsModule::pools(PoolId::DexPremium(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexPlus(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexBonus(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 10000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexExtra(CHFJ_SETT_LP)),
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
			RewardsModule::share_and_withdrawn_reward(PoolId::DexPremium(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexPlus(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexBonus(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexExtra(CHFJ_SETT_LP), ALICE),
			(10000, 0)
		);

		assert_ok!(IncentivesModule::withdraw_dex_share(
			Origin::signed(ALICE),
			CHFJ_SETT_LP,
			8000
		));
		System::assert_last_event(Event::incentives(crate::Event::WithdrawDEXShare(ALICE, CHFJ_SETT_LP, 8000)));
		
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
			RewardsModule::pools(PoolId::DexPremium(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 2000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexPlus(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 2000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexBonus(CHFJ_SETT_LP)),
			PoolInfo {
				total_shares: 2000,
				total_rewards: 0,
				total_withdrawn_rewards: 0
			}
		);
		assert_eq!(
			RewardsModule::pools(PoolId::DexExtra(CHFJ_SETT_LP)),
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
			RewardsModule::share_and_withdrawn_reward(PoolId::DexPremium(CHFJ_SETT_LP), ALICE),
			(2000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexPlus(CHFJ_SETT_LP), ALICE),
			(2000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexBonus(CHFJ_SETT_LP), ALICE),
			(2000, 0)
		);
		assert_eq!(
			RewardsModule::share_and_withdrawn_reward(PoolId::DexExtra(CHFJ_SETT_LP), ALICE),
			(2000, 0)
		);
	});
}

#[test]
fn update_dex_incentive_rewards_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			IncentivesModule::update_dex_incentive_rewards(Origin::signed(ALICE), vec![]),
			BadOrigin
		);
		assert_noop!(
			IncentivesModule::update_dex_incentive_rewards(Origin::signed(4), vec![(DNAR, 200), (SETT, 1000)],),
			Error::<Runtime>::InvalidCurrencyId
		);

		assert_eq!(IncentivesModule::incentive_reward_amount(DNAR_USDJ_LP), 0);

		assert_ok!(IncentivesModule::update_dex_incentive_rewards(
			Origin::signed(4),
			vec![(DNAR_USDJ_LP, 200), (DNAR_SETT_LP, 1000)],
		));
		assert_eq!(IncentivesModule::incentive_reward_amount(DNAR_USDJ_LP), 200);
		assert_eq!(IncentivesModule::incentive_reward_amount(DNAR_SETT_LP), 1000);

		assert_ok!(IncentivesModule::update_dex_incentive_rewards(
			Origin::signed(4),
			vec![(DNAR_SETT_LP, 100), (DNAR_SETT_LP, 300), (DNAR_SETT_LP, 500)],
		));
		assert_eq!(IncentivesModule::incentive_reward_amount(DNAR_SETT_LP), 500);
	});
}

#[test]
fn pay_out_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(TokensModule::deposit(SDEX, &DexIncentivePool::get(), 10000));
		assert_ok!(TokensModule::deposit(SETT, &DexPremiumPool::get(), 10000));
		assert_ok!(TokensModule::deposit(SETT, &DexPlusPool::get(), 10000));
		assert_ok!(TokensModule::deposit(USDJ, &DexBonusPool::get(), 10000));
		assert_ok!(TokensModule::deposit(EURJ, &DexExtraPool::get(), 10000));

		assert_eq!(TokensModule::free_balance(SDEX, &DexIncentivePool::get()), 10000);
		assert_eq!(TokensModule::free_balance(SDEX, &ALICE), 0);
		IncentivesModule::payout(&ALICE, PoolId::DexIncentive(DNAR), 1000);
		assert_eq!(TokensModule::free_balance(SDEX, &DexIncentivePool::get()), 9000);
		assert_eq!(TokensModule::free_balance(SDEX, &ALICE), 1000);

		assert_eq!(TokensModule::free_balance(SETT, &DexPremiumPool::get()), 10000);
		assert_eq!(TokensModule::free_balance(SETT, &ALICE), 0);
		IncentivesModule::payout(&ALICE, PoolId::DexPremium(DNAR), 1000);
		assert_eq!(TokensModule::free_balance(SETT, &DexPremiumPool::get()), 9000);
		assert_eq!(TokensModule::free_balance(SETT, &ALICE), 1000);

		assert_eq!(TokensModule::free_balance(SETT, &DexPlusPool::get()), 10000);
		assert_eq!(TokensModule::free_balance(SETT, &ALICE), 0);
		IncentivesModule::payout(&ALICE, PoolId::DexPlus(DNAR), 1000);
		assert_eq!(TokensModule::free_balance(SETT, &DexPlusPool::get()), 9000);
		assert_eq!(TokensModule::free_balance(SETT, &ALICE), 1000);

		assert_eq!(TokensModule::free_balance(USDJ, &DexBonusPool::get()), 10000);
		assert_eq!(TokensModule::free_balance(USDJ, &ALICE), 0);
		IncentivesModule::payout(&ALICE, PoolId::DexBonus(DNAR), 1000);
		assert_eq!(TokensModule::free_balance(USDJ, &DexBonusPool::get()), 9000);
		assert_eq!(TokensModule::free_balance(USDJ, &ALICE), 1000);

		assert_eq!(TokensModule::free_balance(EURJ, &DexExtraPool::get()), 10000);
		assert_eq!(TokensModule::free_balance(EURJ, &ALICE), 0);
		IncentivesModule::payout(&ALICE, PoolId::DexExtra(DNAR), 1000);
		assert_eq!(TokensModule::free_balance(EURJ, &DexExtraPool::get()), 9000);
		assert_eq!(TokensModule::free_balance(EURJ, &ALICE), 1000);
	});
}

#[test]
fn accumulate_reward_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(IncentivesModule::update_dex_incentive_rewards(
			Origin::signed(4),
			vec![(DNAR_USDJ_LP, 100), (DOT_USDJ_LP, 200),],
		));
		assert_ok!(IncentivesModule::update_dex_premium_rewards(
			Origin::signed(4),
			vec![(DNAR_USDJ_LP, 100), (DOT_USDJ_LP, 200),],
		));
		assert_ok!(IncentivesModule::update_dex_plus_rewards(
			Origin::signed(4),
			vec![(DNAR_USDJ_LP, 100), (DOT_USDJ_LP, 200),],
		));
		assert_ok!(IncentivesModule::update_dex_bonus_rewards(
			Origin::signed(4),
			vec![(DNAR_USDJ_LP, 100), (DOT_USDJ_LP, 200),],
		));
		assert_ok!(IncentivesModule::update_dex_extra_rewards(
			Origin::signed(4),
			vec![(DNAR_USDJ_LP, 100), (DOT_USDJ_LP, 200),],
		));

		assert_eq!(IncentivesModule::accumulate_reward(10, |_, _| {}), vec![]);

		RewardsModule::add_share(&ALICE, PoolId::DexIncentive(DNAR_USDJ_LP), 1);
		RewardsModule::add_share(&ALICE, PoolId::DexPremium(DNAR_USDJ_LP), 1);
		RewardsModule::add_share(&ALICE, PoolId::DexPlus(DNAR_USDJ_LP), 1);
		RewardsModule::add_share(&ALICE, PoolId::DexBonus(DNAR_USDJ_LP), 1);
		RewardsModule::add_share(&ALICE, PoolId::DexExtra(DNAR_USDJ_LP), 1);
		assert_eq!(
			IncentivesModule::accumulate_reward(40, |_, _| {}),
			vec![(SDEX, 3100), (SETT, 3100), (SETT, 3100), (USDJ, 5), (EURJ, 5)]
		);

		RewardsModule::add_share(&ALICE, PoolId::DexIncentive(DNAR_USDJ_LP), 1);
		RewardsModule::add_share(&ALICE, PoolId::DexPremium(DNAR_USDJ_LP), 1);
		RewardsModule::add_share(&ALICE, PoolId::DexPlus(DNAR_USDJ_LP), 1);
		RewardsModule::add_share(&ALICE, PoolId::DexBonus(DNAR_USDJ_LP), 1);
		RewardsModule::add_share(&ALICE, PoolId::DexExtra(DNAR_USDJ_LP), 1);
		assert_eq!(
			IncentivesModule::accumulate_reward(40, |_, _| {}),
			vec![(SDEX, 3300), (SETT, 3300), (SETT, 3300), (USDJ, 9), (EURJ, 9)]
		);

		assert_eq!(IncentivesModule::accumulate_reward(59, |_, _| {}), vec![]);
	});
}
