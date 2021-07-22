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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{log, pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{GetByKey, Happened, MultiCurrency, RewardHandler};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, MaybeDisplay, UniqueSaturatedInto, Zero},
	DispatchResult, FixedPointNumber, RuntimeDebug,
};
use sp_std::{fmt::Debug, vec::Vec};
use support::{SerpTreasury, DEXIncentives, DEXManager, Rate};

mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

/// PoolId for various rewards pools
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum PoolId<AccountId> {
	// TODO: Update new swapped changes
	/// Rewards in Setter (SETT) to market makers who provide Dex liquidity.
	DexIncentive(CurrencyId),

	/// Rewards pool(SetterCurrencyId) (SETT) for market makers who provide Dex liquidity
	/// for SettCurrency (System Stablecoins) pools only.
	DexPremium(CurrencyId),
}

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_rewards::Config<Share = Balance, Balance = Balance, PoolId = PoolId>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The vault account to keep rewards for type DexIncentive PoolId
		/// Receives Setter - SETT/NSETT
		#[pallet::constant]
		type DexIncentivePool: Get<Self::AccountId>;

		/// The vault account to keep rewards for type DexPremium PoolId
		/// Receives Setheum Dirham or Mena - DRAM/MENA
		/// for incentivized (DexIncentivePool) pools only.
		#[pallet::constant]
		type DexPremiumPool: Get<Self::AccountId>;

		/// The DexPremium reward rates (in Rate [_,_]) of specific currencies, by key value pair.
		#[pallet::constant]
		type DexPremiumRewardRates: GetByKey<Self::CurrencyId, Self::Rate>;

		/// The General DexPremium inflation rate (in Balance).
		#[pallet::constant]
		type DexPremiumInflationRate: Get<Balance>;

		/// The Incentive reward type (SETT/NSETT)
		/// SETT in Setheum, NSETT in Neom
		#[pallet::constant]
		type SetterCurrencyId: Get<CurrencyId>;

		/// The Premium reward type (DRAM/MENA)
		/// DRAM in Setheum, MENA in Neom.
		#[pallet::constant]
		type DexerCurrencyId: Get<CurrencyId>;

		/// The Native Currency type (DNAR/NEOM)
		/// DNAR in Setheum, NEOM in Neom
		#[pallet::constant]
		type NativeCurrencyId: Get<CurrencyId>;

		/// The stable currency ids (SettCurrencies)
		type StableCurrencyIds: Get<Vec<CurrencyId>>;

		/// The origin which may update incentives related params
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// The origin which may update incentives Accumulate Period.
		type AccumulatePeriodUpdateOrigin: EnsureOrigin<Self::Origin>;

		/// SERP treasury to issue rewards in stablecoin (Setter (SETT)).
		type SerpTreasury: SerpTreasury<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// Currency to transfer/issue assets
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// Dex to supply liquidity info
		type Dex: DEXManager<Self::AccountId, CurrencyId, Balance>;

		/// The module id, keep DexShare LP.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Share amount is not enough
		NotEnough,
		/// Invalid currency id
		InvalidCurrencyId,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Deposit Dex share. \[who, dex_share_type, deposit_amount\]
		DepositDexShare(T::AccountId, CurrencyId, Balance),
		/// Withdraw Dex share. \[who, dex_share_type, withdraw_amount\]
		WithdrawDexShare(T::AccountId, CurrencyId, Balance),
		/// Claim rewards. \[who, pool_id\]
		ClaimRewards(T::AccountId, T::PoolId),
	}

	/// Mapping from dex liquidity currency type to its Incentive rewards
	/// amount per period
	#[pallet::storage]
	#[pallet::getter(fn dex_incentive_rewards)]
	pub type DexIncentiveRewards<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

	/// Mapping to AccumulatePeriod for reward accumulation
	/// in blocknumber to accumulate rewards periodically.
	/// The period to accumulate rewards.
	#[pallet::storage]
	#[pallet::getter(fn accumulate_period)]
	pub type AccumulatePeriod<T: Config> = StorageMap<_, Twox64Concat, BlockNumber, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::deposit_dex_share())]
		#[transactional]
		pub fn deposit_dex_share(
			origin: OriginFor<T>,
			lp_currency_id: CurrencyId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_deposit_dex_share(&who, lp_currency_id, amount)?;
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::withdraw_dex_share())]
		#[transactional]
		pub fn withdraw_dex_share(
			origin: OriginFor<T>,
			lp_currency_id: CurrencyId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_withdraw_dex_share(&who, lp_currency_id, amount)?;
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::claim_rewards())]
		#[transactional]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			<orml_rewards::Pallet<T>>::claim_rewards(&who, &pool_id);
			Self::deposit_event(Event::ClaimRewards(who, pool_id));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_dex_incentive_rewards(updates.len() as u32))]
		#[transactional]
		pub fn update_dex_incentive_rewards(
			origin: OriginFor<T>,
			updates: Vec<(CurrencyId, Balance)>,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			for (currency_id, amount) in updates {
				ensure!(currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);
				DexIncentiveRewards::<T>::insert(currency_id, amount);
			}
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_accumulate_period(updates.len() as u32))]
		#[transactional]
		pub fn update_accumulate_period(
			origin: OriginFor<T>,
			blocknumber: BlockNumber,
		) -> DispatchResultWithPostInfo {
			T::AccumulatePeriodUpdateOrigin::ensure_origin(origin)?;
			for (blocknumber) in updates {
				AccumulatePeriod::<T>::insert(blocknumber);
			}
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}
}

impl<T: Config> DEXIncentives<T::AccountId, CurrencyId, Balance> for Pallet<T> {
	fn dex_premium_rewards(lp_currency_id: CurrencyId) -> Balance {
		ensure!(
			lp_currency_id.is_dex_share_currency_id(),
			Error::<T>::InvalidCurrencyId
		);
		ensure!(
			T::DexPremiumPools::get().contains(&lp_currency_id),
			Error::<T>::InvalidCurrencyId,
		);
		let (reward_rate_den, reward_rate_num) = T::DexPremiumRewardRates::get(lp_currency_id);
		let pool_reward_rate = U256 = U256::from(reward_rate_num).saturating_div(U256::from(reward_rate_den));
		let inflation_rate = U256 = U256::from(T::DexPremiumInflationRate::get());
		rewards = &inflation_rate.saturating_mul(&pool_reward_rate)
			.and_then(|n| TryInto::<Balance>::try_into(n)
			.ok())
			.unwrap_or_else(Zero::zero);
	}

	fn do_deposit_dex_share(who: &T::AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		ensure!(lp_currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);

		T::Currency::transfer(lp_currency_id, who, &Self::account_id(), amount)?;
		<orml_rewards::Pallet<T>>::add_share(
			who,
			PoolId::DexIncentive(lp_currency_id),
			amount.unique_saturated_into(),
		);
		<orml_rewards::Pallet<T>>::add_share(
			who,
			PoolId::DexPremium(lp_currency_id),
			amount.unique_saturated_into(),
		);
		Self::deposit_event(Event::DepositDexShare(who.clone(), lp_currency_id, amount));
		Ok(())
	}

	fn do_withdraw_dex_share(who: &T::AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		ensure!(lp_currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);
		ensure!(
			<orml_rewards::Pallet<T>>::share_and_withdrawn_reward(
				PoolId::DexIncentive(lp_currency_id), &who
			).0 >= amount && <orml_rewards::Pallet<T>>::share_and_withdrawn_reward(
				PoolId::DexPremium(lp_currency_id), &who
			).0 >= amount,
			Error::<T>::NotEnough,
		);

		T::Currency::transfer(lp_currency_id, &Self::account_id(), &who, amount)?;
		<orml_rewards::Pallet<T>>::remove_share(
			who,
			PoolId::DexIncentive(lp_currency_id),
			amount.unique_saturated_into(),
		);
		<orml_rewards::Pallet<T>>::remove_share(
			who,
			PoolId::DexPremium(lp_currency_id),
			amount.unique_saturated_into(),
		);
		Self::deposit_event(Event::WithdrawDexShare(who.clone(), lp_currency_id, amount));
		Ok(())
	}
}

impl<T: Config> RewardHandler<T::AccountId> for Pallet<T> {
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type PoolId = T::PoolId;
	type Share = Balance;

	fn accumulate_reward(now: T::BlockNumber, mut callback: impl FnMut(PoolId, Balance)) -> Vec<(CurrencyId, Balance)> {
		let mut accumulated_rewards: Vec<(CurrencyId, Balance)> = vec![];

		// accumulate reward periodically
		let accumulation_period = Self::accumulate_period();
		if now % accumulation_period == Zero::zero() {
			let mut accumulated_incentive: Balance = Zero::zero();
			let mut accumulated_premium: Balance = Zero::zero();
			let incentive_currency_id = T::SetterCurrencyId::get();
			let premium_currency_id = T::DexerCurrencyId::get();

			for (pool_id, pool_info) in orml_rewards::Pools::<T>::iter() {
				if !pool_info.total_shares.is_zero() {
					match pool_id {
						PoolId::DexIncentive(_) => {
							let incentive_reward = Self::dex_incentive_rewards(pool_id.clone());

							/// issue Dex Incentive Currency
							if !incentive_reward.is_zero()
								&& T::SerpTreasury::issue_setter(
									&T::DexIncentivePool::get(),
									incentive_reward,
								)
								.is_ok()
							{
								callback(pool_id, incentive_reward);
								accumulated_incentive = accumulated_incentive.saturating_add(incentive_reward);
							}
						}

						PoolId::DexPremium(_) => {
							let premium_reward = Self::dex_premium_rewards(pool_id.clone());

							/// issue Dex Premium Currency
							if !premium_reward.is_zero()
								&& T::SerpTreasury::issue_dexer(
									&T::DexPremiumPool::get(),
									premium_reward,
								)
								.is_ok()
							{
								callback(pool_id, premium_reward);
								accumulated_premium = accumulated_premium.saturating_add(premium_reward);
							}
						}
					}
				}
			}
			if !accumulated_incentive.is_zero() {
				accumulated_rewards.push((incentive_currency_id, accumulated_incentive));
			}
			if !accumulated_premium.is_zero() {
				accumulated_rewards.push((premium_currency_id, accumulated_premium));
			}
		}

		accumulated_rewards
	}
	
	fn payout(who: &T::AccountId, pool_id: PoolId, amount: Balance) {
		let (pool_account, currency_id) = match pool_id {
			PoolId::DexIncentive(_) => (T::DexIncentivePool::get(), T::SetterCurrencyId::get()),
			PoolId::DexPremium(_) => (T::DexPremiumPool::get(), T::DexerCurrencyId::get()),
		};

		// payout the reward to user from the pool. it should not affect the
		// process, ignore the result to continue. if it fails, just the user will not
		// be rewarded, there will not be increase in user balance.
		let _ = T::Currency::transfer(&currency_id, &pool_account, &who, amount);
	}
}
