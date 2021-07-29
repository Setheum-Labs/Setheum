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
use orml_traits::{Happened, MultiCurrency, RewardHandler};
use primitives::{AccountId, Amount, Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, MaybeDisplay, One, UniqueSaturatedInto, Zero},
	DispatchResult, FixedPointNumber, RuntimeDebug,
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
	// TODO: Update new swapped changes
	/// Rewards in Setter (SETT) to market makers who provide Dex liquidity.
	DexIncentive(CurrencyId),

	/// Rewards pool(SetterCurrencyId) (SETT) for market makers who provide Dex liquidity
	/// for SettCurrency (System Stablecoins) pools only.
	DexBonus(CurrencyId),
}

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_rewards::Config<Share = Balance, Balance = Balance, PoolId = PoolId<AccountId>>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Incentive reward type (SETT)
		/// SETT in Setheum.
		#[pallet::constant]
		type SetterCurrencyId: Get<CurrencyId>;

		/// The Bonus reward type (USDJ)
		/// USDJ in Setheum.
		#[pallet::constant]
		type GetSettUSDCurrencyId: Get<CurrencyId>;

		/// The Native Currency type (DNAR)
		/// DNAR in Setheum.
		#[pallet::constant]
		type DirhamCurrencyId: Get<CurrencyId>;

		/// The Native Currency type (DNAR)
		/// DNAR in Setheum.
		#[pallet::constant]
		type NativeCurrencyId: Get<CurrencyId>;

		/// The stable currency ids (SettCurrencies)
		type StableCurrencyIds: Get<Vec<CurrencyId>>;

		/// The period to accumulate rewards
		type AccumulatePeriod: Get<Self::BlockNumber>;

		/// The origin which may update incentives related params
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

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
		/// Invalid pool id
		InvalidPoolId,
		/// Invalid rate
		InvalidRate,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Deposit Dex share. \[who, dex_share_type, deposit_amount\]
		DepositDexShare(T::AccountId, CurrencyId, Balance),
		/// Withdraw Dex share. \[who, dex_share_type, withdraw_amount\]
		WithdrawDexShare(T::AccountId, CurrencyId, Balance),
		/// Payout rewards. \[who, pool_id, reward_currency_type, actual_payout, deduction_amount\]
		PayoutRewards(
			T::AccountId,
			PoolId<AccountId>,
			CurrencyId,
			Balance,
			Balance,
		),
		/// Incentive reward amount updated. \[pool_id, reward_amount_per_period\]
		IncentiveRewardAmountUpdated(PoolId<AccountId>, Balance),
		/// Bonus reward amount updated. \[pool_id, reward_amount_per_period\]
		BonusRewardAmountUpdated(PoolId<AccountId>, Rate),
		/// Payout deduction rate updated. \[pool_id, deduction_rate\]
		PayoutDeductionRateUpdated(PoolId<AccountId>, Rate),
	}

	/// Mapping from pool to its fixed reward amount per period.
	///
	/// IncentiveRewardAmount: map PoolId => Balance
	#[pallet::storage]
	#[pallet::getter(fn incentive_reward_amount)]
	pub type IncentiveRewardAmount<T: Config> = 
		StorageMap<_, Twox64Concat, PoolId<AccountId>, Balance, ValueQuery>;

	/// Mapping from pool to its fixed reward amount per period.
	///
	/// BonusRewardAmount: map PoolId => Balance
	#[pallet::storage]
	#[pallet::getter(fn bonus_reward_amount)]
	pub type BonusRewardAmount<T: Config> = 
		StorageMap<_, Twox64Concat, PoolId<AccountId>, Balance, ValueQuery>;

	/// Mapping from pool to its payout deduction rate.
	///
	/// PayoutDeductionRates: map PoolId => Rate
	#[pallet::storage]
	#[pallet::getter(fn payout_deduction_rates)]
	pub type PayoutDeductionRates<T: Config> =
		StorageMap<_, Twox64Concat, PoolId<AccountId>, Rate, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {

			// accumulate reward periodically
			if now % T::AccumulatePeriod::get() == Zero::zero() {
				let mut count: u32 = 0;
				let incentive_currency_id = T::SetterCurrencyId::get();
				let bonus_currency_id = T::GetSettUSDCurrencyId::get();

				for (pool_id, pool_info) in orml_rewards::Pools::<T>::iter() {
					if !pool_info.total_shares.is_zero() {
						match pool_id {
							PoolId::DexIncentive(_) => {
								count += 1;
								let incentive_reward = Self::incentive_reward_amount(pool_id.clone());

								/// issue Dex Incentive Currency
								if !incentive_reward.is_zero() {
									let res = T::Currency::deposit(
										incentive_currency_id,
										&Self::account_id(),
										incentive_reward,
									)?;
									match res {
										Ok(_) => {
											<orml_rewards::Pallet<T>>::accumulate_reward(
												&pool_id,
												incentive_reward,
											);
										}
										Err(e) => {
											log::warn!(
												target: "incentives",
												"transfer: failed to issue {:?} {:?} to {:?}: {:?}. \
												This is unexpected but should be safe",
												incentive_reward, incentive_currency_id, Self::account_id(), e
											);
										}
									}
								}
							}

							PoolId::DexBonus(_) => {
								count += 1;
								let bonus_reward = Self::bonus_reward_amount(pool_id.clone());

								/// issue Dex Incentive Currency
								if !bonus_reward.is_zero() {
									let res = T::Currency::deposit(
										bonus_currency_id,
										&Self::account_id(),
										bonus_reward,
									)?;
									match res {
										Ok(_) => {
											<orml_rewards::Pallet<T>>::accumulate_reward(
												&pool_id,
												bonus_reward,
											);
										}
										Err(e) => {
											log::warn!(
												target: "incentives",
												"transfer: failed to issue {:?} {:?} to {:?}: {:?}. \
												This is unexpected but should be safe",
												bonus_reward, bonus_currency_id, Self::account_id(), e
											);
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
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_deposit_dex_share(&who, lp_currency_id, amount)?;
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::withdraw_dex_share())]
		#[transactional]
		pub fn withdraw_dex_share(
			origin: OriginFor<T>,
			lp_currency_id: CurrencyId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_withdraw_dex_share(&who, lp_currency_id, amount)?;
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::claim_rewards())]
		#[transactional]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			pool_id: PoolId<AccountId>
		) ->  DispatchResult {
			let who = ensure_signed(origin)?;
			<orml_rewards::Pallet<T>>::claim_rewards(&who, &pool_id);
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_incentive_rewards(updates.len() as u32))]
		#[transactional]
		pub fn update_incentive_rewards(
			origin: OriginFor<T>,
			updates: Vec<(PoolId<AccountId>, Balance)>,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
			for (pool_id, amount) in updates {
				match pool_id {
					PoolId::DexIncentive(currency_id) => {
						ensure!(currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);
					}
					_ => {
						return Err(Error::<T>::InvalidPoolId.into());
					}
				}
				IncentiveRewardAmount::<T>::insert(&pool_id, amount);
				Self::deposit_event(Event::IncentiveRewardAmountUpdated(pool_id, amount));
			}
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_bonus_reward_amount(updates.len() as u32))]
		#[transactional]
		pub fn update_bonus_reward_amount(
			origin: OriginFor<T>,
			updates: Vec<(PoolId<AccountId>, Balance)>,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			for (pool_id, amount) in updates {
				match pool_id {
					PoolId::DexBonus(currency_id) => {
						ensure!(currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);
					}
					_ => {
						return Err(Error::<T>::InvalidPoolId.into());
					}
				}
				BonusRewardAmount::<T>::insert(&pool_id, amount);
				Self::deposit_event(Event::BonusRewardAmountUpdated(pool_id, amount));
			}
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_payout_deduction_rates(updates.len() as u32))]
		#[transactional]
		pub fn update_payout_deduction_rates(
			origin: OriginFor<T>,
			updates: Vec<(PoolId<AccountId>, Rate)>,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
			for (pool_id, deduction_rate) in updates {
				match pool_id {
					PoolId::DexIncentive(currency_id) => {
						ensure!(currency_id.is_dex_share_currency_id(), Error::<T>::InvalidCurrencyId);
					}
					_ => {}
				}
				ensure!(deduction_rate <= Rate::one(), Error::<T>::InvalidRate);
				PayoutDeductionRates::<T>::insert(&pool_id, deduction_rate);
				Self::deposit_event(Event::PayoutDeductionRateUpdated(pool_id, deduction_rate));
			}
			Ok(())
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
			PoolId::DexIncentive(lp_currency_id),
			amount.unique_saturated_into(),
		);
		<orml_rewards::Pallet<T>>::add_share(
			who,
			PoolId::DexBonus(lp_currency_id),
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
				PoolId::DexBonus(lp_currency_id), &who
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
			PoolId::DexBonus(lp_currency_id),
			amount.unique_saturated_into(),
		);
		Self::deposit_event(Event::WithdrawDexShare(who.clone(), lp_currency_id, amount));
		Ok(())
	}
}

impl<T: Config> RewardHandler<T::AccountId> for Pallet<T> {
	type Balance = Balance;
	type PoolId = PoolId<AccountId>;

	fn payout(who: &T::AccountId, pool_id: &Self::PoolId, payout_amount: Self::Balance) {
		if payout_amount.is_zero() {
			return;
		}

		let currency_id = match pool_id {
			PoolId::DexIncentive(_) => T::SetterCurrencyId::get(),
			PoolId::DexBonus(_) => T::GetSettUSDCurrencyId::get(),
		};

		// calculate actual payout and deduction amount
		let (actual_payout, deduction_amount) = {
			let deduction_amount = Self::payout_deduction_rates(pool_id)
				.saturating_mul_int(payout_amount)
				.min(payout_amount);
			if !deduction_amount.is_zero() {
				// re-accumulate deduction to rewards pool if deduction amount is not zero
				<orml_rewards::Pallet<T>>::accumulate_reward(pool_id, deduction_amount);
			}
			(payout_amount.saturating_sub(deduction_amount), deduction_amount)
		};

		// payout the reward(exclude deduction) to user from the pool. it should not affect the
		// process, ignore the result to continue. if it fails, just the user will not
		// be rewarded, there will not increase user balance.
		let res = T::Currency::transfer(currency_id, &Self::account_id(), &who, actual_payout);
		if let Err(e) = res {
			log::warn!(
				target: "incentives",
				"transfer: failed to transfer {:?} {:?} from {:?} to {:?}: {:?}. \
				This is unexpected but should be safe",
				actual_payout, currency_id, Self::account_id(), who, e
			);
			debug_assert!(false);
		}

		Self::deposit_event(Event::PayoutRewards(
			who.clone(),
			pool_id.clone(),
			currency_id,
			actual_payout,
			deduction_amount,
		));
	}
}
