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

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::DispatchResult,
    traits::Get
};
use frame_system::{self as system, ensure_signed};

#[cfg(feature = "std")]
use serde::Deserialize;
use sp_runtime::{RuntimeDebug, DispatchError};
use sp_std::prelude::*;

use module_support::moderation::IsAccountBlocked;
use slixon_permissions::ChannelPermission;
use slixon_posts::{Module as Posts, Post, PostById};
use slixon_channels::Module as Channels;
use slixon_utils::{Error as UtilsError, remove_from_vec, WhoAndWhen, PostId};

pub mod rpc;

pub type ReactionId = u64;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Deserialize))]
#[cfg_attr(feature = "std", serde(untagged))]
pub enum ReactionKind {
    Upvote,
    Downvote,
}

impl Default for ReactionKind {
    fn default() -> Self {
        ReactionKind::Upvote
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Reaction<T: Trait> {

    /// Unique sequential identifier of a reaction. Examples of reaction ids: `1`, `2`, `3`,
    /// and so on.
    pub id: ReactionId,

    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,
    pub kind: ReactionKind,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + slixon_utils::Trait
    + slixon_posts::Trait
    + slixon_channels::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type PostReactionScores: PostReactionScores<Self>;
}

pub const FIRST_REACTION_ID: u64 = 1;

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ReactionsModule {

        /// The next reaction id.
        pub NextReactionId get(fn next_reaction_id): ReactionId = FIRST_REACTION_ID;

        pub ReactionById get(fn reaction_by_id):
            map hasher(twox_64_concat) ReactionId => Option<Reaction<T>>;

        pub ReactionIdsByPostId get(fn reaction_ids_by_post_id):
            map hasher(twox_64_concat) PostId => Vec<ReactionId>;

        pub PostReactionIdByAccount get(fn post_reaction_id_by_account):
            map hasher(twox_64_concat) (T::AccountId, PostId) => ReactionId;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        PostReactionCreated(AccountId, PostId, ReactionId),
        PostReactionUpdated(AccountId, PostId, ReactionId),
        PostReactionDeleted(AccountId, PostId, ReactionId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Reaction was not found by id.
        ReactionNotFound,
        /// Account has already reacted to this post/comment.
        AccountAlreadyReacted,
        /// There is no reaction by account on this post/comment.
        ReactionByAccountNotFound,
        /// Only reaction owner can update their reaction.
        NotReactionOwner,
        /// New reaction kind is the same as old one on this post/comment.
        SameReaction,

        /// Not allowed to react on a post/comment in a hidden channel.
        CannotReactWhenChannelHidden,
        /// Not allowed to react on a post/comment if a root post is hidden.
        CannotReactWhenPostHidden,

        /// User has no permission to upvote posts/comments in this channel.
        NoPermissionToUpvote,
        /// User has no permission to downvote posts/comments in this channel.
        NoPermissionToDownvote,
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 10_000 + T::DbWeight::get().reads_writes(6, 5)]
    pub fn create_post_reaction(origin, post_id: PostId, kind: ReactionKind) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      let post = &mut Posts::require_post(post_id)?;
      ensure!(
        !<PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
        Error::<T>::AccountAlreadyReacted
      );

      let channel = post.get_channel()?;
      ensure!(!channel.hidden, Error::<T>::CannotReactWhenChannelHidden);
      ensure!(Posts::<T>::is_root_post_visible(post_id)?, Error::<T>::CannotReactWhenPostHidden);

      ensure!(T::IsAccountBlocked::is_allowed_account(owner.clone(), channel.id), UtilsError::<T>::AccountIsBlocked);

      let reaction_id = Self::insert_new_reaction(owner.clone(), kind);

      match kind {
        ReactionKind::Upvote => {
          Channels::ensure_account_has_channel_permission(
            owner.clone(),
            &post.get_channel()?,
            ChannelPermission::Upvote,
            Error::<T>::NoPermissionToUpvote.into()
          )?;
          post.inc_upvotes();
        },
        ReactionKind::Downvote => {
          Channels::ensure_account_has_channel_permission(
            owner.clone(),
            &post.get_channel()?,
            ChannelPermission::Downvote,
            Error::<T>::NoPermissionToDownvote.into()
          )?;
          post.inc_downvotes();
        }
      }

      if post.is_owner(&owner) {
        <PostById<T>>::insert(post_id, post.clone());
      }

      T::PostReactionScores::score_post_on_reaction(owner.clone(), post, kind)?;

      ReactionIdsByPostId::mutate(post.id, |ids| ids.push(reaction_id));
      <PostReactionIdByAccount<T>>::insert((owner.clone(), post_id), reaction_id);

      Self::deposit_event(RawEvent::PostReactionCreated(owner, post_id, reaction_id));
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(3, 2)]
    pub fn update_post_reaction(origin, post_id: PostId, reaction_id: ReactionId, new_kind: ReactionKind) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
        Error::<T>::ReactionByAccountNotFound
      );

      let mut reaction = Self::require_reaction(reaction_id)?;
      let post = &mut Posts::require_post(post_id)?;

      ensure!(owner == reaction.created.account, Error::<T>::NotReactionOwner);
      ensure!(reaction.kind != new_kind, Error::<T>::SameReaction);

      if let Some(channel_id) = post.try_get_channel_id() {
        ensure!(T::IsAccountBlocked::is_allowed_account(owner.clone(), channel_id), UtilsError::<T>::AccountIsBlocked);
      }

      let old_kind = reaction.kind;
      reaction.kind = new_kind;
      reaction.updated = Some(WhoAndWhen::<T>::new(owner.clone()));

      match new_kind {
        ReactionKind::Upvote => {
          post.inc_upvotes();
          post.dec_downvotes();
        },
        ReactionKind::Downvote => {
          post.inc_downvotes();
          post.dec_upvotes();
        },
      }

      T::PostReactionScores::score_post_on_reaction(owner.clone(), post, old_kind)?;
      T::PostReactionScores::score_post_on_reaction(owner.clone(), post, new_kind)?;

      <ReactionById<T>>::insert(reaction_id, reaction);
      <PostById<T>>::insert(post_id, post);

      Self::deposit_event(RawEvent::PostReactionUpdated(owner, post_id, reaction_id));
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(4, 4)]
    pub fn delete_post_reaction(origin, post_id: PostId, reaction_id: ReactionId) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      ensure!(
        <PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
        Error::<T>::ReactionByAccountNotFound
      );

      // TODO extract Self::require_reaction(reaction_id)?;
      let reaction = Self::require_reaction(reaction_id)?;
      let post = &mut Posts::require_post(post_id)?;

      ensure!(owner == reaction.created.account, Error::<T>::NotReactionOwner);
      if let Some(channel_id) = post.try_get_channel_id() {
        ensure!(T::IsAccountBlocked::is_allowed_account(owner.clone(), channel_id), UtilsError::<T>::AccountIsBlocked);
      }

      match reaction.kind {
        ReactionKind::Upvote => post.dec_upvotes(),
        ReactionKind::Downvote => post.dec_downvotes(),
      }

      T::PostReactionScores::score_post_on_reaction(owner.clone(), post, reaction.kind)?;

      <PostById<T>>::insert(post_id, post.clone());
      <ReactionById<T>>::remove(reaction_id);
      ReactionIdsByPostId::mutate(post.id, |ids| remove_from_vec(ids, reaction_id));
      <PostReactionIdByAccount<T>>::remove((owner.clone(), post_id));

      Self::deposit_event(RawEvent::PostReactionDeleted(owner, post_id, reaction_id));
      Ok(())
    }
  }
}

impl<T: Trait> Module<T> {
    // FIXME: don't add reaction in storage before the checks in 'create_reaction' are done
    pub fn insert_new_reaction(account: T::AccountId, kind: ReactionKind) -> ReactionId {
        let id = Self::next_reaction_id();
        let reaction: Reaction<T> = Reaction {
            id,
            created: WhoAndWhen::<T>::new(account),
            updated: None,
            kind,
        };

        <ReactionById<T>>::insert(id, reaction);
        NextReactionId::mutate(|n| { *n += 1; });

        id
    }

    /// Get `Reaction` by id from the storage or return `ReactionNotFound` error.
    pub fn require_reaction(reaction_id: ReactionId) -> Result<Reaction<T>, DispatchError> {
        Ok(Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?)
    }
}

/// Handler that will be called right before the post reaction is toggled.
pub trait PostReactionScores<T: Trait> {
    fn score_post_on_reaction(actor: T::AccountId, post: &mut Post<T>, reaction_kind: ReactionKind) -> DispatchResult;
}

impl<T: Trait> PostReactionScores<T> for () {
    fn score_post_on_reaction(_actor: T::AccountId, _post: &mut Post<T>, _reaction_kind: ReactionKind) -> DispatchResult {
        Ok(())
    }
}
