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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{log, pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{Happened, MultiCurrency, RewardHandler};
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

	/// Rewards pool(DexCurrencyId) (SDEX or HALAL) for market makers who provide Dex
	/// liquidity
	DexIncentive(CurrencyId),

	/// Rewards pool(SetterCurrencyId) for LPs who provide Dex liquidity
	/// for Setter (SETT) pools only.
	DexSetterReward(CurrencyId),
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

		/// The period to accumulate rewards
		#[pallet::constant]
		type AccumulatePeriod: Get<Self::BlockNumber>;

		/// The reward type for incentive.
		#[pallet::constant]
		type DexCurrencyId: Get<CurrencyId>;

		/// The reward type for `dex_setter`, rename to `SetterCurrencyId`.
		#[pallet::constant]
		type StableCurrencyId: Get<CurrencyId>;

		/// The vault account to keep rewards.
		#[pallet::constant]
		type RewardsVaultAccountId: Get<Self::AccountId>;

		/// The origin which may update incentive related params
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// SERP treasury to issue rewards in stablecoin (Setter (SETT)).
		type SerpTreasury: SerpTreasury<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// Currency to transfer/issue assets
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// DEX to supply liquidity info
		type DEX: DEXManager<Self::AccountId, CurrencyId, Balance>;

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
		/// Invalid pool id
		InvalidPoolId,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Deposit DEX share. \[who, dex_share_type, deposit_amount\]
		DepositDexShare(T::AccountId, CurrencyId, Balance),
		/// Withdraw DEX share. \[who, dex_share_type, withdraw_amount\]
		WithdrawDexShare(T::AccountId, CurrencyId, Balance),
		/// Claim rewards. \[who, pool_id\]
		ClaimRewards(T::AccountId, T::PoolId),
	}

	/// Mapping from pool to its fixed reward amount per period.
	///
	/// IncentiveRewardAmount: map PoolId => Balance
	#[pallet::storage]
	#[pallet::getter(fn incentive_reward_amount)]
	pub type IncentiveRewardAmount<T: Config> =
		StorageMap<_, Twox64Concat, PoolId, Balance, ValueQuery>;

	/// Mapping from pool to its fixed reward rate per period.
	///
	/// DexSetterRewardRate: map PoolId => Rate
	#[pallet::storage]
	#[pallet::getter(fn dex_setter_reward_rate)]
	pub type DexSetterRewardRate<T: Config> =
		StorageMap<_, Twox64Concat, PoolId, Rate, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			// accumulate reward periodically
			if now % T::AccumulatePeriod::get() == Zero::zero() {
				let mut count: u32 = 0;
				let stable_currency_id = T::StableCurrencyId::get();

				for (pool_id, pool_info) in orml_rewards::Pools::<T>::iter() {
					if !pool_info.total_shares.is_zero() {
						match pool_id {
							PoolId::DexIncentive(_) => {
								count += 1;
								let incentive_reward_amount = Self::incentive_reward_amount(pool_id.clone());

								/// issue SettinDex (SDEX).
								if !incentive_reward_amount.is_zero() {
									let res = T::SerpTreasury::issue_dexer(
										&T::RewardsVaultAccountId::get(),
										incentive_reward_amount,
										false,
									);
									match res {
										Ok(_) => {
											<orml_rewards::Pallet<T>>::accumulate_reward(
												&pool_id,
												incentive_reward_amount,
											);
										}
										Err(e) => {
											log::warn!(
												target: "incentives",
												"issue_dexer: failed to issue {:?} dexer to {:?}: {:?}. \
														This is unexpected but should be safe",
												incentive_reward_amount, T::RewardsVaultAccountId::get(), e
											);
										}
									}
								}
							}

							PoolId::DexSetterReward(lp_currency_id) => {
								count += 1;
								let dex_setter_reward_rate = Self::dex_setter_reward_rate(pool_id.clone());

								if !dex_setter_reward_rate.is_zero() {
									if let Some((currency_id_a, currency_id_b)) =
										lp_currency_id.split_dex_share_currency_id()
									{
										// accumulate dex_setter reward only for liquidity pool of stable currency id
										let dex_setter_reward_base = if currency_id_a == stable_currency_id {
											T::DEX::get_liquidity_pool(stable_currency_id, currency_id_b).0
										} else if currency_id_b == stable_currency_id {
											T::DEX::get_liquidity_pool(stable_currency_id, currency_id_a).0
										} else {
											Zero::zero()
										};
										let dex_setter_reward_amount =
											dex_setter_reward_rate.saturating_mul_int(dex_setter_reward_base);

										// issue Setter (SETT).
										if !dex_setter_reward_amount.is_zero() {
											let res = T::SerpTreasury::issue_standard( // update to `issue_reserve`
												&T::RewardsVaultAccountId::get(),
												dex_setter_reward_amount,
												false,
											);
											match res {
												Ok(_) => {
													<orml_rewards::Pallet<T>>::accumulate_reward(
														&pool_id,
														dex_setter_reward_amount,
													);
												}
												Err(e) => {
													log::warn!(
														target: "incentives",
														"issue_standard: failed to issue {:?} Setter to {:?}: {:?}. \
														This is unexpected but should be safe",
														dex_setter_reward_amount, T::RewardsVaultAccountId::get(), e
													);
												}
											}
										}
									}
								}
							}

							_ => {}
						}
					}
				}

				T::WeightInfo::on_initialize(count)
			} else {
				0
			}
		}
	}

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

		#[pallet::weight(<T as Config>::WeightInfo::update_incentive_rewards(updates.len() as u32))]
		#[transactional]
		pub fn update_incentive_rewards(
			origin: OriginFor<T>,
			updates: Vec<(T::PoolId, Balance)>,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			for (pool_id, amount) in updates {
				if pool_id == PoolId::DexIncentive(currency_id) {
					ensure!(currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);
				} else {
					return Err(Error::<T>::InvalidPoolId.into());
				}
				IncentiveRewardAmount::<T>::insert(pool_id, amount);
			}
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_dex_setter_rewards(updates.len() as u32))]
		#[transactional]
		pub fn update_dex_setter_rewards(
			origin: OriginFor<T>,
			updates: Vec<(T::PoolId, Rate)>,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			for (pool_id, rate) in updates {
				if pool_id == PoolId::DexSetterReward(currency_id) {
						ensure!(currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);
				} else {
					return Err(Error::<T>::InvalidPoolId.into());
				}
				DexSetterRewardRate::<T>::insert(pool_id, rate);
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
	fn do_deposit_dex_share(who: &T::AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		ensure!(lp_currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);

		T::Currency::transfer(lp_currency_id, who, &Self::account_id(), amount)?;
		<orml_rewards::Pallet<T>>::add_share(
			who,
			&PoolId::DexIncentive(lp_currency_id),
			amount.unique_saturated_into(),
		);
		<orml_rewards::Pallet<T>>::add_share(who, &PoolId::DexSetterReward(lp_currency_id), amount);

		Self::deposit_event(Event::DepositDexShare(who.clone(), lp_currency_id, amount));
		Ok(())
	}

	fn do_withdraw_dex_share(who: &T::AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		ensure!(lp_currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);
		ensure!(
			<orml_rewards::Pallet<T>>::share_and_withdrawn_reward(&PoolId::DexIncentive(lp_currency_id), &who).0
				>= amount && <orml_rewards::Pallet<T>>::share_and_withdrawn_reward(
				&PoolId::DexSetterReward(lp_currency_id),
				&who
			)
			.0 >= amount,
			Error::<T>::NotEnough,
		);

		T::Currency::transfer(lp_currency_id, &Self::account_id(), &who, amount)?;
		<orml_rewards::Pallet<T>>::remove_share(
			who,
			&PoolId::DexIncentive(lp_currency_id),
			amount.unique_saturated_into(),
		);
		<orml_rewards::Pallet<T>>::remove_share(who, &PoolId::DexSetterReward(lp_currency_id), amount);

		Self::deposit_event(Event::WithdrawDexShare(who.clone(), lp_currency_id, amount));
		Ok(())
	}
}

pub struct OnUpdateLoan<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> Happened<(T::AccountId, CurrencyId, Amount, Balance)> for OnUpdateLoan<T> {
	fn happened(info: &(T::AccountId, CurrencyId, Amount, Balance)) {
		let (who, currency_id, adjustment, previous_amount) = info;
		let adjustment_abs =
			sp_std::convert::TryInto::<Balance>::try_into(adjustment.saturating_abs()).unwrap_or_default();

		if !adjustment_abs.is_zero() {
			let new_share_amount = if adjustment.is_positive() {
				previous_amount.saturating_add(adjustment_abs)
			} else {
				previous_amount.saturating_sub(adjustment_abs)
			};

			<orml_rewards::Pallet<T>>::set_share(who, &PoolId::StandardIncentive(*currency_id), new_share_amount);
		}
	}
}

impl<T: Config> RewardHandler<T::AccountId> for Pallet<T> {
	type Balance = Balance;
	type PoolId = T::PoolId;

	fn payout(who: &T::AccountId, pool_id: &Self::PoolId, amount: Self::Balance) {
		let currency_id = match pool_id {
			PoolId::DexIncentive(_) | PoolId::DexSetterReward(_) => T::StableCurrencyId::get(), /// TODO: Update to `T::SetterCurrencyId::get()`.
		};

		// payout the reward to user from the pool. it should not affect the
		// process, ignore the result to continue. if it fails, just the user will not
		// be rewarded, there will not be increase in user balance.
		let res = T::Currency::transfer(currency_id, &T::RewardsVaultAccountId::get(), &who, amount);
		if let Err(e) = res {
			log::warn!(
				target: "incentives",
				"transfer: failed to transfer {:?} {:?} from {:?} to {:?}: {:?}. \
				This is unexpected but should be safe",
				amount, currency_id, T::RewardsVaultAccountId::get(), who, e
			);
			debug_assert!(false);
		}
	}
}
