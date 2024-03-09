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

use crate::Rate;
use parity_scale_codec::{Decode, Encode};
use primitives::CurrencyId;
use scale_info::TypeInfo;
use sp_runtime::{DispatchResult, RuntimeDebug};
use sp_std::prelude::*;

/// PoolId for various rewards pools
// Pool types:
// 1. EcdpSetrLiquidityRewards: record the shares and rewards for Setter (SETR)) ECDP users who are staking LP tokens.
// 2. EcdpUssdLiquidityRewards: record the shares and rewards for Slick USD (USSD) ECDP users who are staking LP tokens.
// 3. EdfisLiquidityRewards: record the shares and rewards for Edfis makers who are staking LP token.
// 4. EdfisXLiquidityRewards: record the shares and rewards for Edfis X (Cross-chain) makers who are staking LP token.
// 5. MoyaEarnRewards: record the shares and rewards for users of Moya Earn (Moya Liquid Staking Protocol).
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum PoolId {
	/// Rewards and shares pool for Setter (SETR)) ECDP users who are staking LP token(LPCurrencyId)
	EcdpSetrLiquidityRewards(CurrencyId),

	/// Rewards and shares pool for Slick USD (USSD) ECDP users who are staking LP token(LPCurrencyId)
	EcdpUssdLiquidityRewards(CurrencyId),

	/// Rewards and shares pool for Edfis market makers who stake LP token(LPCurrencyId)
	EdfisLiquidityRewards(CurrencyId),

	/// Rewards and shares pool for Edfis X (Cross-chain) market makers who stake LP token(LPCurrencyId)
	EdfisXLiquidityRewards(CurrencyId),

	/// Rewards and shares pool for Moya Earn
	MoyaEarnRewards(CurrencyId),
}

pub trait IncentivesManager<AccountId, Balance, CurrencyId, PoolId> {
	/// Gets reward amount for the given reward currency added per period
	fn get_incentive_reward_amount(pool_id: PoolId, currency_id: CurrencyId) -> Balance;
	/// Stake LP token to add shares to pool
	fn deposit_edfis_share(who: &AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult;
	/// Unstake LP token to remove shares from pool
	fn withdraw_edfis_share(who: &AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult;
	/// Claim all available rewards for specific `PoolId`
	fn claim_rewards(who: AccountId, pool_id: PoolId) -> DispatchResult;
	/// Gets deduction reate for claiming reward early
	fn get_claim_reward_deduction_rate(pool_id: PoolId) -> Rate;
	/// Gets the pending rewards for a pool, for an account
	fn get_pending_rewards(pool_id: PoolId, who: AccountId, reward_currency: Vec<CurrencyId>) -> Vec<Balance>;
}

pub trait Incentives<AccountId, CurrencyId, Balance> {
	fn do_deposit_edfis_share(who: &AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult;
	fn do_withdraw_edfis_share(who: &AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult;
}

#[cfg(feature = "std")]
impl<AccountId, CurrencyId, Balance> Incentives<AccountId, CurrencyId, Balance> for () {
	fn do_deposit_edfis_share(_: &AccountId, _: CurrencyId, _: Balance) -> DispatchResult {
		Ok(())
	}

	fn do_withdraw_edfis_share(_: &AccountId, _: CurrencyId, _: Balance) -> DispatchResult {
		Ok(())
	}
}
