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

//! # Moderation Module
//!
//! The Moderation module allows any user (account) to report an account, channel, post or even
//! IPFS CID, if they think it's a spam, abuse or inappropriate for a specific channel.
//!
//! Moderators of a channel can review reported entities and suggest a moderation status for them:
//! `Block` or `Allowed`. A channel owner can make a final decision: either block or allow any entity
//! within the channel they control.
//!
//! This pallet also has a setting to auto-block the content after a specific number of statuses
//! from moderators that suggest to block the entity. If the entity is added to allow list,
//! then the entity cannot be blocked.
//!
//! The next rules applied to the blocked entities:
//!
//! - A post cannot be added to a channel if an IPFS CID of this post is blocked in this channel.
//! - An account cannot create posts in a channel if this account is blocked in this channel.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, ensure,
    dispatch::DispatchResult,
    traits::Get,
};
use frame_system::{self as system, ensure_signed};

use slixon_utils::{Content, WhoAndWhen, ChannelId, Module as Utils, PostId};
use slixon_channels::Module as Channels;

// TODO: move all tests to slixon-integration-tests
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod functions;

pub type ReportId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum EntityId<AccountId> {
    Content(Content),
    Account(AccountId),
    Channel(ChannelId),
    Post(PostId),
}

/// Entity status is used in two cases: when moderators suggest a moderation status
/// for a reported entity; or when a channel owner makes a final decision to either block
/// or allow this entity within the channel.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum EntityStatus {
    Allowed,
    Blocked,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Report<T: Trait> {
    id: ReportId,
    created: WhoAndWhen<T>,
    /// An id of reported entity: account, channel, post or IPFS CID.
    reported_entity: EntityId<T::AccountId>,
    /// Within what channel (scope) this entity has been reported.
    reported_within: ChannelId, // TODO rename: reported_in_channel
    /// A reason should describe why this entity should be blocked in this channel.
    reason: Content,
}

// TODO rename to SuggestedEntityStatus
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SuggestedStatus<T: Trait> {
    /// An account id of a moderator who suggested this status.
    suggested: WhoAndWhen<T>,
    /// `None` if a moderator wants to signal that they have reviewed the entity,
    /// but they are not sure about what status should be applied to it.
    status: Option<EntityStatus>,
    /// `None` if a suggested status is not based on any reports.
    report_id: Option<ReportId>,
}

// TODO rename to ModerationSettings?
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ChannelModerationSettings {
    autoblock_threshold: Option<u16>
}

// TODO rename to ModerationSettingsUpdate?
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ChannelModerationSettingsUpdate {
    pub autoblock_threshold: Option<Option<u16>>
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + slixon_posts::Trait
    + slixon_channels::Trait
    + pallet_channel_follows::Trait
    + slixon_utils::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type DefaultAutoblockThreshold: Get<u16>;
}

pub const FIRST_REPORT_ID: u64 = 1;

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ModerationModule {

        /// The next moderation report id.
        pub NextReportId get(fn next_report_id): ReportId = FIRST_REPORT_ID;

        /// Report details by its id (key).
        pub ReportById get(fn report_by_id):
            map hasher(twox_64_concat) ReportId
            => Option<Report<T>>;

        /// Report id if entity (key 1) was reported by a specific account (key 2)
        pub ReportIdByAccount get(fn report_id_by_account):
            map hasher(twox_64_concat) (EntityId<T::AccountId>, T::AccountId)
            => Option<ReportId>;

        /// Ids of all reports in this channel (key).
        pub ReportIdsByChannelId get(fn report_ids_by_channel_id):
            map hasher(twox_64_concat) ChannelId
            => Vec<ReportId>;

        /// Ids of all reports related to a specific entity (key 1) sent to this channel (key 2).
        pub ReportIdsByEntityInChannel get(fn report_ids_by_entity_in_channel): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) ChannelId
            => Vec<ReportId>;

        /// An entity (key 1) status (`Blocked` or `Allowed`) in this channel (key 2).
        pub StatusByEntityInChannel get(fn status_by_entity_in_channel): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) ChannelId
            => Option<EntityStatus>;

        /// Entity (key 1) statuses suggested by channel (key 2) moderators.
        pub SuggestedStatusesByEntityInChannel get(fn suggested_statuses): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) ChannelId
            => Vec<SuggestedStatus<T>>;

        /// A custom moderation settings for a certain channel (key).
        pub ModerationSettings get(fn moderation_settings):
            map hasher(twox_64_concat) ChannelId
            => Option<ChannelModerationSettings>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId,
        EntityId = EntityId<<T as system::Trait>::AccountId>
    {
        EntityReported(AccountId, ChannelId, EntityId, ReportId),
        EntityStatusSuggested(AccountId, ChannelId, EntityId, Option<EntityStatus>),
        EntityStatusUpdated(AccountId, ChannelId, EntityId, Option<EntityStatus>),
        EntityStatusDeleted(AccountId, ChannelId, EntityId),
        ModerationSettingsUpdated(AccountId, ChannelId),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The account has already reported this entity.
        AlreadyReportedEntity,
        /// The entity has no status in this channel. Nothing to delete.
        EntityHasNoStatusInScope,
        /// Entity scope differs from the scope provided.
        EntityNotInScope,
        /// Entity was not found by its id.
        EntityNotFound,
        /// Entity status is already as suggested one.
        SuggestedSameEntityStatus,
        /// Provided entity scope does not exist.
        ScopeNotFound,
        /// Account does not have a permission to suggest a new entity status.
        NoPermissionToSuggestEntityStatus,
        /// Account does not have a permission to update an entity status.
        NoPermissionToUpdateEntityStatus,
        /// Account does not have a permission to update the moderation settings.
        NoPermissionToUpdateModerationSettings,
        /// No updates provided for the channel settings.
        NoUpdatesForModerationSettings,
        /// Report reason should not be empty.
        ReasonIsEmpty,
        /// Report was not found by its id.
        ReportNotFound,
        /// Trying to suggest an entity status in a scope that is different from the scope
        /// the entity was reported in.
        SuggestedStatusInWrongScope,
        /// Entity status has already been suggested by this moderator account.
        AlreadySuggestedEntityStatus,
    }
}

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        const DefaultAutoblockThreshold: u16 = T::DefaultAutoblockThreshold::get();

        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        /// Report any entity by any person with mandatory reason.
        /// `entity` scope and the `scope` provided mustn't differ
        #[weight = 10_000 + T::DbWeight::get().reads_writes(6, 5)]
        pub fn report_entity(
            origin,
            entity: EntityId<T::AccountId>,
            scope: ChannelId,
            reason: Content
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO check this func, if looks strange
            Utils::<T>::ensure_content_is_some(&reason).map_err(|_| Error::<T>::ReasonIsEmpty)?;
            
            Utils::<T>::is_valid_content(reason.clone())?;

            ensure!(Channels::<T>::require_channel(scope).is_ok(), Error::<T>::ScopeNotFound);
            Self::ensure_entity_in_scope(&entity, scope)?;

            let not_reported_yet = Self::report_id_by_account((&entity, &who)).is_none();
            ensure!(not_reported_yet, Error::<T>::AlreadyReportedEntity);

            let report_id = Self::next_report_id();
            let new_report = Report::<T>::new(report_id, who.clone(), entity.clone(), scope, reason);

            ReportById::<T>::insert(report_id, new_report);
            ReportIdByAccount::<T>::insert((&entity, &who), report_id);
            ReportIdsByChannelId::mutate(scope, |ids| ids.push(report_id));
            ReportIdsByEntityInChannel::<T>::mutate(&entity, scope, |ids| ids.push(report_id));
            NextReportId::mutate(|n| { *n += 1; });

            Self::deposit_event(RawEvent::EntityReported(who, scope, entity, report_id));
            Ok(())
        }

        /// Leave a feedback on the report either it's confirmation or ignore.
        /// `origin` - any permitted account (e.g. Channel owner or moderator that's set via role)
        #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
        pub fn suggest_entity_status(
            origin,
            entity: EntityId<T::AccountId>,
            scope: ChannelId, // TODO make scope as Option, but either scope or report_id_opt should be Some
            status: Option<EntityStatus>,
            report_id_opt: Option<ReportId>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if let Some(report_id) = report_id_opt {
                let report = Self::require_report(report_id)?;
                ensure!(scope == report.reported_within, Error::<T>::SuggestedStatusInWrongScope);
            }

            let entity_status = StatusByEntityInChannel::<T>::get(&entity, scope);
            ensure!(!(entity_status.is_some() && status == entity_status), Error::<T>::SuggestedSameEntityStatus);

            let channel = Channels::<T>::require_channel(scope).map_err(|_| Error::<T>::ScopeNotFound)?;
            Channels::<T>::ensure_account_has_channel_permission(
                who.clone(),
                &channel,
                slixon_permissions::ChannelPermission::SuggestEntityStatus,
                Error::<T>::NoPermissionToSuggestEntityStatus.into(),
            )?;

            let mut suggestions = SuggestedStatusesByEntityInChannel::<T>::get(&entity, scope);
            let is_already_suggested = suggestions.iter().any(|suggestion| suggestion.suggested.account == who);
            ensure!(!is_already_suggested, Error::<T>::AlreadySuggestedEntityStatus);
            suggestions.push(SuggestedStatus::new(who.clone(), status.clone(), report_id_opt));

            let block_suggestions_total = suggestions.iter()
                .filter(|suggestion| suggestion.status == Some(EntityStatus::Blocked))
                .count();

            let autoblock_threshold_opt = Self::moderation_settings(scope)
                .unwrap_or_else(Self::default_autoblock_threshold_as_settings)
                .autoblock_threshold;

            if let Some(autoblock_threshold) = autoblock_threshold_opt {
                if block_suggestions_total >= autoblock_threshold as usize {
                    Self::block_entity_in_scope(&entity, scope)?;
                }
            }

            SuggestedStatusesByEntityInChannel::<T>::insert(entity.clone(), scope, suggestions);

            Self::deposit_event(RawEvent::EntityStatusSuggested(who, scope, entity, status));
            Ok(())
        }

        /// Allows a channel owner/admin to update the final moderation status of a reported entity.
        #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
        pub fn update_entity_status(
            origin,
            entity: EntityId<T::AccountId>,
            scope: ChannelId,
            status_opt: Option<EntityStatus>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO: add `forbid_content` parameter and track entity Content blocking via OCW
            //  - `forbid_content` - whether to block `Content` provided with entity.

            let channel = Channels::<T>::require_channel(scope).map_err(|_| Error::<T>::ScopeNotFound)?;
            Self::ensure_account_status_manager(who.clone(), &channel)?;

            if let Some(status) = &status_opt {
                let is_entity_in_scope = Self::ensure_entity_in_scope(&entity, scope).is_ok();

                if is_entity_in_scope && status == &EntityStatus::Blocked {
                    Self::block_entity_in_scope(&entity, scope)?;
                } else {
                    StatusByEntityInChannel::<T>::insert(entity.clone(), scope, status);
                }
            } else {
                StatusByEntityInChannel::<T>::remove(entity.clone(), scope);
            }

            Self::deposit_event(RawEvent::EntityStatusUpdated(who, scope, entity, status_opt));
            Ok(())
        }

        /// Allows a channel owner/admin to delete a current status of a reported entity.
        #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
        pub fn delete_entity_status(
            origin,
            entity: EntityId<T::AccountId>,
            scope: ChannelId
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let status = Self::status_by_entity_in_channel(&entity, scope);
            ensure!(status.is_some(), Error::<T>::EntityHasNoStatusInScope);

            let channel = Channels::<T>::require_channel(scope).map_err(|_| Error::<T>::ScopeNotFound)?;
            Self::ensure_account_status_manager(who.clone(), &channel)?;

            StatusByEntityInChannel::<T>::remove(&entity, scope);

            Self::deposit_event(RawEvent::EntityStatusDeleted(who, scope, entity));
            Ok(())
        }

        // todo: add ability to delete report_ids

        // TODO rename to update_settings?
        #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
        fn update_moderation_settings(
            origin,
            channel_id: ChannelId,
            update: ChannelModerationSettingsUpdate
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let has_updates = update.autoblock_threshold.is_some();
            ensure!(has_updates, Error::<T>::NoUpdatesForModerationSettings);

            let channel = Channels::<T>::require_channel(channel_id)?;

            Channels::<T>::ensure_account_has_channel_permission(
                who.clone(),
                &channel,
                slixon_permissions::ChannelPermission::UpdateChannelSettings,
                Error::<T>::NoPermissionToUpdateModerationSettings.into(),
            )?;

            // `true` if there is at least one updated field.
            let mut should_update = false;

            let mut settings = Self::moderation_settings(channel_id)
                .unwrap_or_else(Self::default_autoblock_threshold_as_settings);

            if let Some(autoblock_threshold) = update.autoblock_threshold {
                if autoblock_threshold != settings.autoblock_threshold {
                    settings.autoblock_threshold = autoblock_threshold;
                    should_update = true;
                }
            }

            if should_update {
                ModerationSettings::insert(channel_id, settings);
                Self::deposit_event(RawEvent::ModerationSettingsUpdated(who, channel_id));
            }
            Ok(())
        }
    }
}
