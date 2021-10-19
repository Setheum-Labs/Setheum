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

use crate::*;

use sp_runtime::traits::Saturating;
use pallet_sudo::Module as Sudo;
use frame_support::{
    dispatch::DispatchError,
    traits::schedule::DispatchTime,
};

use slixon_permissions::ChannelPermission;
use slixon_channels::Channel;

impl<T: Trait> Module<T> {
    pub fn require_plan(plan_id: SubscriptionPlanId) -> Result<SubscriptionPlan<T>, DispatchError> {
        Ok(Self::plan_by_id(plan_id).ok_or(Error::<T>::SubscriptionPlanNotFound)?)
    }

    pub fn require_subscription(subscription_id: SubscriptionId) -> Result<Subscription<T>, DispatchError> {
        Ok(Self::subscription_by_id(subscription_id).ok_or(Error::<T>::SubscriptionNotFound)?)
    }

    pub fn ensure_subscriptions_manager(account: T::AccountId, channel: &Channel<T>) -> DispatchResult {
        Channels::<T>::ensure_account_has_channel_permission(
            account,
            channel,
            ChannelPermission::UpdateChannelSettings,
            Error::<T>::NoPermissionToUpdateSubscriptionPlan.into()
        )
    }

    pub fn get_period_in_blocks(period: SubscriptionPeriod<T::BlockNumber>) -> T::BlockNumber {
        match period {
            SubscriptionPeriod::Daily => T::DailyPeriodInBlocks::get(),
            SubscriptionPeriod::Weekly => T::WeeklyPeriodInBlocks::get(),
            SubscriptionPeriod::Monthly => T::MonthlyPeriodInBlocks::get(),
            SubscriptionPeriod::Quarterly => T::QuarterlyPeriodInBlocks::get(),
            SubscriptionPeriod::Yearly => T::YearlyPeriodInBlocks::get(),
            SubscriptionPeriod::Custom(block_number) => block_number,
        }
    }

    pub(crate) fn schedule_recurring_subscription_payment(
        subscription_id: SubscriptionId,
        period: SubscriptionPeriod<T::BlockNumber>
    ) -> DispatchResult {
        let period_in_blocks = Self::get_period_in_blocks(period);
        let task_name = (SUBSCRIPTIONS_ID, subscription_id).encode();
        let when = <system::Module<T>>::block_number().saturating_add(period_in_blocks);

        T::Scheduler::schedule_named(
            task_name,
            DispatchTime::At(when),
            Some((period_in_blocks, 12)),
            1,
            frame_system::RawOrigin::Signed(Sudo::<T>::key()).into(),
            Call::process_subscription_payment(subscription_id).into()
        ).map_err(|_| Error::<T>::CannotScheduleReccurentPayment)?;
        Ok(())
    }

    pub(crate) fn cancel_recurring_subscription_payment(subscription_id: SubscriptionId) {
        let _ = T::Scheduler::cancel_named((SUBSCRIPTIONS_ID, subscription_id).encode())
            .map_err(|_| Error::<T>::RecurringPaymentMissing);
        // todo: emmit event with status
    }

    pub(crate) fn do_unsubscribe(who: T::AccountId, subscription: &mut Subscription<T>) -> DispatchResult {
        let channel_id = Self::require_plan(subscription.plan_id)?.channel_id;
        let subscription_id = subscription.id;

        Self::cancel_recurring_subscription_payment(subscription_id);
        subscription.is_active = false;

        SubscriptionById::<T>::insert(subscription_id, subscription);
        SubscriptionIdsByPatron::<T>::mutate(who, |ids| remove_from_vec(ids, subscription_id));
        SubscriptionIdsByChannel::mutate(channel_id, |ids| remove_from_vec(ids, subscription_id));

        Ok(())
    }

    pub(crate) fn filter_subscriptions_by_plan(
        subscription_id: SubscriptionId,
        plan_id: SubscriptionPlanId
    ) -> bool {
        if let Ok(subscription) = Self::require_subscription(subscription_id) {
            return subscription.plan_id == plan_id;
        }
        false
    }
}

impl<T: Trait> SubscriptionPlan<T> {
    pub fn new(
        id: SubscriptionPlanId,
        created_by: T::AccountId,
        channel_id: ChannelId,
        wallet: Option<T::AccountId>,
        price: BalanceOf<T>,
        period: SubscriptionPeriod<T::BlockNumber>,
        content: Content
    ) -> Self {
        Self {
            id,
            created: WhoAndWhen::<T>::new(created_by),
            updated: None,
            is_active: true,
            content,
            channel_id,
            wallet,
            price,
            period,
        }
    }

    pub fn try_get_recipient(&self) -> Option<T::AccountId> {
        self.wallet.clone()
            .or_else(|| Module::<T>::recipient_wallet(self.channel_id))
            .or_else(|| {
                Channels::<T>::require_channel(self.channel_id).map(|channel| channel.owner).ok()
            })
    }
}

impl<T: Trait> Subscription<T> {
    pub fn new(
        id: SubscriptionId,
        created_by: T::AccountId,
        wallet: Option<T::AccountId>,
        plan_id: SubscriptionPlanId
    ) -> Self {
        Self {
            id,
            created: WhoAndWhen::<T>::new(created_by),
            updated: None,
            is_active: true,
            wallet,
            plan_id,
        }
    }

    pub fn ensure_subscriber(&self, who: &T::AccountId) -> DispatchResult {
        ensure!(&self.created.account == who, Error::<T>::NotSubscriber);
        Ok(())
    }
}