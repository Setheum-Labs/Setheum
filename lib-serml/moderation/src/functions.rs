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

use frame_support::dispatch::DispatchError;
use slixon_posts::Module as Posts;
use slixon_channels::Channel;
use slixon_channel_follows::Module as ChannelFollows;
use module_support::moderation::*;

impl<T: Trait> Module<T> {
    pub fn require_report(report_id: ReportId) -> Result<Report<T>, DispatchError> {
        Ok(Self::report_by_id(report_id).ok_or(Error::<T>::ReportNotFound)?)
    }

    /// Get entity channel_id if it exists.
    /// Content and Account has no scope, consider check with `if let Some`
    fn get_entity_scope(entity: &EntityId<T::AccountId>) -> Result<Option<ChannelId>, DispatchError> {
        match entity {
            EntityId::Content(content) => {
                Utils::<T>::ensure_content_is_some(content).map(|_| None)
            },
            EntityId::Account(_) => Ok(None),
            EntityId::Channel(channel_id) => {
                let channel = Channels::<T>::require_channel(*channel_id)?;
                let root_channel_id = channel.try_get_parent()?;

                Ok(Some(root_channel_id))
            },
            EntityId::Post(post_id) => {
                let post = Posts::<T>::require_post(*post_id)?;
                let channel_id = post.get_channel()?.id;

                Ok(Some(channel_id))
            },
        }
    }

    #[allow(dead_code)]
    // fixme: do we need this?
    fn ensure_entity_exists(entity: &EntityId<T::AccountId>) -> DispatchResult {
        match entity {
            EntityId::Content(content) => Utils::<T>::ensure_content_is_some(content),
            EntityId::Account(_) => Ok(()),
            EntityId::Channel(channel_id) => Channels::<T>::ensure_channel_exists(*channel_id),
            EntityId::Post(post_id) => Posts::<T>::ensure_post_exists(*post_id),
        }.map_err(|_| Error::<T>::EntityNotFound.into())
    }

    pub(crate) fn block_entity_in_scope(entity: &EntityId<T::AccountId>, scope: ChannelId) -> DispatchResult {
        // TODO: update counters, when entity is moved
        // TODO: think, what and where we should change something if entity is moved
        match entity {
            EntityId::Content(_) => (),
            EntityId::Account(account_id)
                => ChannelFollows::<T>::unfollow_channel_by_account(account_id.clone(), scope)?,
            EntityId::Channel(channel_id) => Channels::<T>::try_move_channel_to_root(*channel_id)?,
            EntityId::Post(post_id) => Posts::<T>::delete_post_from_channel(*post_id)?,
        }
        StatusByEntityInChannel::<T>::insert(entity, scope, EntityStatus::Blocked);
        Ok(())
    }

    pub(crate) fn ensure_account_status_manager(who: T::AccountId, channel: &Channel<T>) -> DispatchResult {
        Channels::<T>::ensure_account_has_channel_permission(
            who,
            &channel,
            slixon_permissions::ChannelPermission::UpdateEntityStatus,
            Error::<T>::NoPermissionToUpdateEntityStatus.into(),
        )
    }

    pub(crate) fn ensure_entity_in_scope(entity: &EntityId<T::AccountId>, scope: ChannelId) -> DispatchResult {
        if let Some(entity_scope) = Self::get_entity_scope(entity)? {
            ensure!(entity_scope == scope, Error::<T>::EntityNotInScope);
        }
        Ok(())
    }

    pub fn default_autoblock_threshold_as_settings() -> ChannelModerationSettings {
        ChannelModerationSettings {
            autoblock_threshold: Some(T::DefaultAutoblockThreshold::get())
        }
    }
}

impl<T: Trait> Report<T> {
    pub fn new(
        id: ReportId,
        created_by: T::AccountId,
        reported_entity: EntityId<T::AccountId>,
        scope: ChannelId,
        reason: Content
    ) -> Self {
        Self {
            id,
            created: WhoAndWhen::<T>::new(created_by),
            reported_entity,
            reported_within: scope,
            reason
        }
    }
}

impl<T: Trait> SuggestedStatus<T> {
    pub fn new(who: T::AccountId, status: Option<EntityStatus>, report_id: Option<ReportId>) -> Self {
        Self {
            suggested: WhoAndWhen::<T>::new(who),
            status,
            report_id
        }
    }
}

// TODO: maybe simplify using one common trait?
impl<T: Trait> IsAccountBlocked<T::AccountId> for Module<T> {
    fn is_blocked_account(account: T::AccountId, scope: ChannelId) -> bool {
        let entity = EntityId::Account(account);

        Self::status_by_entity_in_channel(entity, scope) == Some(EntityStatus::Blocked)
    }

    fn is_allowed_account(account: T::AccountId, scope: ChannelId) -> bool {
        let entity = EntityId::Account(account);

        Self::status_by_entity_in_channel(entity, scope) != Some(EntityStatus::Blocked)
    }
}

impl<T: Trait> IsChannelBlocked for Module<T> {
    fn is_blocked_channel(channel_id: ChannelId, scope: ChannelId) -> bool {
        let entity = EntityId::Channel(channel_id);

        Self::status_by_entity_in_channel(entity, scope) == Some(EntityStatus::Blocked)
    }

    fn is_allowed_channel(channel_id: ChannelId, scope: ChannelId) -> bool {
        let entity = EntityId::Channel(channel_id);

        Self::status_by_entity_in_channel(entity, scope) != Some(EntityStatus::Blocked)
    }
}

impl<T: Trait> IsPostBlocked<PostId> for Module<T> {
    fn is_blocked_post(post_id: PostId, scope: ChannelId) -> bool {
        let entity = EntityId::Post(post_id);

        Self::status_by_entity_in_channel(entity, scope) == Some(EntityStatus::Blocked)
    }

    fn is_allowed_post(post_id: PostId, scope: ChannelId) -> bool {
        let entity = EntityId::Post(post_id);

        Self::status_by_entity_in_channel(entity, scope) != Some(EntityStatus::Blocked)
    }
}

impl<T: Trait> IsContentBlocked for Module<T> {
    fn is_blocked_content(content: Content, scope: ChannelId) -> bool {
        let entity = EntityId::Content(content);

        Self::status_by_entity_in_channel(entity, scope) == Some(EntityStatus::Blocked)
    }

    fn is_allowed_content(content: Content, scope: ChannelId) -> bool {
        let entity = EntityId::Content(content);

        Self::status_by_entity_in_channel(entity, scope) != Some(EntityStatus::Blocked)
    }
}
