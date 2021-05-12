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

use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use orml_traits::{Happened, MultiCurrency, RewardHandler};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, MaybeDisplay, UniqueSaturatedInto, Zero},
	DispatchResult, FixedPointNumber, ModuleId, RuntimeDebug,
};
use sp_std::{fmt::Debug, vec::Vec};
use support::{DEXIncentives, DEXManager, Rate};

mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

/// PoolId for various rewards pools
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum PoolId<AccountId> {
	/// Rewards pool(NativeCurrencyId) for market makers who provide dex
	/// liquidity
	DexIncentive(CurrencyId),
}

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_rewards::Config<Share = Balance, Balance = Balance, PoolId = PoolId<Self::AccountId>>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The type of validator account id on relaychain.
		type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;

		/// The period to accumulate rewards
		#[pallet::constant]
		type AccumulatePeriod: Get<Self::BlockNumber>;

		/// The reward type for incentive.
		#[pallet::constant]
		type NativeCurrencyId: Get<CurrencyId>;

		/// The reward type for dex saving.
		#[pallet::constant]
		type StableCurrencyId: Get<CurrencyId>;

		/// The vault account to keep rewards.
		#[pallet::constant]
		type RewardsVaultAccountId: Get<Self::AccountId>;

		/// The origin which may update incentive related params
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// Currency for transfer/issue assets
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// DEX to supply liquidity info
		type DEX: DEXManager<Self::AccountId, CurrencyId, Balance>;

		/// The module id, keep DEXShare LP.
		#[pallet::constant]
		type ModuleId: Get<ModuleId>;

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
		DepositDEXShare(T::AccountId, CurrencyId, Balance),
		/// Withdraw DEX share. \[who, dex_share_type, withdraw_amount\]
		WithdrawDEXShare(T::AccountId, CurrencyId, Balance),
	}

	/// Mapping from pool to its fixed reward amount per period.
	#[pallet::storage]
	#[pallet::getter(fn incentive_reward_amount)]
	pub type IncentiveRewardAmount<T: Config> =
		StorageMap<_, Twox64Concat, PoolId<T::AccountId>, Balance, ValueQuery>;

	/// Mapping from pool to its fixed reward rate per period.
	#[pallet::storage]
	#[pallet::getter(fn dex_saving_reward_rate)]
	pub type DexSavingRewardRate<T: Config> =
		StorageMap<_, Twox64Concat, PoolId<T::AccountId>, Rate, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			// accumulate reward periodically
			if now % T::AccumulatePeriod::get() == Zero::zero() {
				let mut count: u32 = 0;
				let native_currency_id = T::NativeCurrencyId::get();
				let stable_currency_id = T::StableCurrencyId::get();

				for (pool_id, pool_info) in orml_rewards::Pools::<T>::iter() {
					if !pool_info.total_shares.is_zero() {
                        PoolId::DexIncentive(_) {
                            count += 1;
                            let incentive_reward_amount = Self::incentive_reward_amount(pool_id.clone());

                            // TODO: Issue DexCurrency instead of NativeCurrency.
                            if !incentive_reward_amount.is_zero()
                                && T::Currency::deposit(
                                    native_currency_id,
                                    &T::RewardsVaultAccountId::get(),
                                    incentive_reward_amount,
                                )
                                .is_ok()
                            {
                                <orml_rewards::Pallet<T>>::accumulate_reward(&pool_id, incentive_reward_amount);
                            }
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

		//TODO: Turn this match statement into a simple for loop and remove `AbhaIncentive`
		#[pallet::weight(<T as Config>::WeightInfo::update_incentive_rewards(updates.len() as u32))]
		#[transactional]
		pub fn update_incentive_rewards(
			origin: OriginFor<T>,
			updates: Vec<(PoolId<T::AccountId>, Balance)>,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			for (pool_id, amount) in updates {
				PoolId::DexIncentive(currency_id) {
					ensure!(currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);
				} else {
					return Err(Error::<T>::InvalidPoolId.into());
				}

				IncentiveRewardAmount::<T>::insert(pool_id, amount);
			}
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
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
		<orml_rewards::Pallet<T>>::add_share(who, &PoolId::DexIncentive(lp_currency_id), amount);

		Self::deposit_event(Event::DepositDEXShare(who.clone(), lp_currency_id, amount));
		Ok(())
	}

	fn do_withdraw_dex_share(who: &T::AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		ensure!(lp_currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);
		ensure!(
			<orml_rewards::Pallet<T>>::share_and_withdrawn_reward(&PoolId::DexIncentive(lp_currency_id), &who).0
				>= amount && <orml_rewards::Pallet<T>>::share_and_withdrawn_reward(
				&PoolId::DexIncentive(lp_currency_id),
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
		<orml_rewards::Pallet<T>>::remove_share(who, &PoolId::DexIncentive(lp_currency_id), amount);

		Self::deposit_event(Event::WithdrawDEXShare(who.clone(), lp_currency_id, amount));
		Ok(())
	}
}

//TODO: Change the `PoolId::DexIncentive(_) => T::NativeCurrencyId::get()` to vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
//TODO: ^^^^^^^^^^^`PoolId::DexIncentive(_) => T::DexCurrencyId::get()` and make `DexCurrencyId = SDEX in Setheum and HDEX in Neom.

impl<T: Config> RewardHandler<T::AccountId> for Pallet<T> {
	type Balance = Balance;
	type PoolId = PoolId<T::DexIncentive>;

	fn payout(who: &T::AccountId, pool_id: &Self::PoolId, amount: Self::Balance) {
		let currency_id = T::NativeCurrencyId::get() {
			&PoolId::DexIncentive(_) = currency_id,
		};

		// payout the reward to user from the pool. it should not affect the
		// process, ignore the result to continue. if it fails, just the user will not
		// be rewarded, there will not increase user balance.
		let _ = T::Currency::transfer(currency_id, &T::RewardsVaultAccountId::get(), &who, amount);
	}
}
