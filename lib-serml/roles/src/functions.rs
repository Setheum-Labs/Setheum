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

use super::*;

use frame_support::dispatch::DispatchError;
use slixon_permissions::ChannelPermissionsContext;

impl<T: Trait> Module<T> {

  /// Check that there is a `Role` with such `role_id` in the storage
  /// or return`RoleNotFound` error.
  pub fn ensure_role_exists(role_id: RoleId) -> DispatchResult {
      ensure!(<RoleById<T>>::contains_key(role_id), Error::<T>::RoleNotFound);
      Ok(())
  }

  /// Get `Role` by id from the storage or return `RoleNotFound` error.
  pub fn require_role(role_id: ChannelId) -> Result<Role<T>, DispatchError> {
      Ok(Self::role_by_id(role_id).ok_or(Error::<T>::RoleNotFound)?)
  }

  /// Ensure that this account is not blocked and has 'ManageRoles' permission in a given channel
  pub fn ensure_role_manager(account: T::AccountId, channel_id: ChannelId) -> DispatchResult {
    ensure!(
      T::IsAccountBlocked::is_allowed_account(account.clone(), channel_id),
      UtilsError::<T>::AccountIsBlocked
    );
    Self::ensure_user_has_channel_permission_with_load_channel(
      User::Account(account),
      channel_id,
      ChannelPermission::ManageRoles,
      Error::<T>::NoPermissionToManageRoles.into()
    )
  }

  fn ensure_user_has_channel_permission_with_load_channel(
    user: User<T::AccountId>,
    channel_id: ChannelId,
    permission: ChannelPermission,
    error: DispatchError,
  ) -> DispatchResult {

    let channel = T::Channels::get_channel(channel_id)?;

    let mut is_owner = false;
    let mut is_follower = false;

    match &user {
      User::Account(account) => {
        is_owner = *account == channel.owner;

        // No need to check if a user is follower, if they already are an owner:
        is_follower = is_owner || T::ChannelFollows::is_channel_follower(account.clone(), channel_id);
      }
      User::Channel(_) => (/* Not implemented yet. */),
    }

    Self::ensure_user_has_channel_permission(
      user,
      ChannelPermissionsContext {
        channel_id,
        is_channel_owner: is_owner,
        is_channel_follower: is_follower,
        channel_perms: channel.permissions
      },
      permission,
      error
    )
  }

  fn ensure_user_has_channel_permission(
    user: User<T::AccountId>,
    ctx: ChannelPermissionsContext,
    permission: ChannelPermission,
    error: DispatchError,
  ) -> DispatchResult {

    match Permissions::<T>::has_user_a_channel_permission(
      ctx.clone(),
      permission.clone()
    ) {
      Some(true) => return Ok(()),
      Some(false) => return Err(error),
      _ => (/* Need to check in dynamic roles */)
    }

    Self::has_permission_in_channel_roles(
      user,
      ctx.channel_id,
      permission,
      error
    )
  }

  fn has_permission_in_channel_roles(
    user: User<T::AccountId>,
    channel_id: ChannelId,
    permission: ChannelPermission,
    error: DispatchError,
  ) -> DispatchResult {

    let role_ids = Self::role_ids_by_user_in_channel(user, channel_id);

    for role_id in role_ids {
      if let Some(role) = Self::role_by_id(role_id) {
        if role.disabled {
          continue;
        }

        let mut is_expired = false;
        if let Some(expires_at) = role.expires_at {
          if expires_at <= <system::Module<T>>::block_number() {
            is_expired = true;
          }
        }

        if !is_expired && role.permissions.contains(&permission) {
          return Ok(());
        }
      }
    }

    Err(error)
  }
}

impl<T: Trait> Role<T> {

  pub fn new(
    created_by: T::AccountId,
    channel_id: ChannelId,
    time_to_live: Option<T::BlockNumber>,
    content: Content,
    permissions: BTreeSet<ChannelPermission>,
  ) -> Result<Self, DispatchError> {

    let role_id = Module::<T>::next_role_id();

    let mut expires_at: Option<T::BlockNumber> = None;
    if let Some(ttl) = time_to_live {
      expires_at = Some(ttl + <system::Module<T>>::block_number());
    }

    let new_role = Role::<T> {
      created: WhoAndWhen::new(created_by),
      updated: None,
      id: role_id,
      channel_id,
      disabled: false,
      expires_at,
      content,
      permissions,
    };

    Ok(new_role)
  }

  pub fn set_disabled(&mut self, disable: bool) -> DispatchResult {
    if self.disabled && disable {
      return Err(Error::<T>::RoleAlreadyDisabled.into());
    } else if !self.disabled && !disable {
      return Err(Error::<T>::RoleAlreadyEnabled.into());
    }

    self.disabled = disable;

    Ok(())
  }

  pub fn revoke_from_users(&self, users: Vec<User<T::AccountId>>) {
    let mut users_by_role = <UsersByRoleId<T>>::take(self.id);

    for user in users.iter() {
      let role_idx_by_user_opt = Module::<T>::role_ids_by_user_in_channel(&user, self.channel_id).iter()
        .position(|x| { *x == self.id });

      if let Some(role_idx) = role_idx_by_user_opt {
        <RoleIdsByUserInChannel<T>>::mutate(user, self.channel_id, |n| { n.swap_remove(role_idx) });
      }

      let user_idx_by_role_opt = users_by_role.iter().position(|x| { x == user });

      if let Some(user_idx) = user_idx_by_role_opt {
        users_by_role.swap_remove(user_idx);
      }
    }
    <UsersByRoleId<T>>::insert(self.id, users_by_role);
  }
}

impl<T: Trait> PermissionChecker for Module<T> {
  type AccountId = T::AccountId;

  fn ensure_user_has_channel_permission(
    user: User<Self::AccountId>,
    ctx: ChannelPermissionsContext,
    permission: ChannelPermission,
    error: DispatchError,
  ) -> DispatchResult {

    Self::ensure_user_has_channel_permission(
      user,
      ctx,
      permission,
      error
    )
  }
}
