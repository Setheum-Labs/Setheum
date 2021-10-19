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

//! # Posts Module
//!
//! Posts are the second crucial component of Subsocial after Channels. This module allows you to 
//! create, update, move (between channels), and hide posts as well as manage owner(s).
//! 
//! Posts can be compared to existing entities on web 2.0 platforms such as:
//! - Posts on Facebook,
//! - Tweets on Twitter,
//! - Images on Instagram,
//! - Articles on Medium,
//! - Shared links on Reddit,
//! - Questions and answers on Stack Overflow.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, fail,
    dispatch::{DispatchError, DispatchResult}, ensure, traits::Get,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use module_support::moderation::{IsAccountBlocked, IsContentBlocked, IsPostBlocked};
use slixon_permissions::ChannelPermission;
use slixon_channels::{Module as Channels, Channel, ChannelById};
use slixon_utils::{
    Module as Utils, Error as UtilsError,
    ChannelId, WhoAndWhen, Content, PostId
};

pub mod functions;

pub mod rpc;

/// Information about a post's owner, its' related channel, content, and visibility.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Post<T: Trait> {

    /// Unique sequential identifier of a post. Examples of post ids: `1`, `2`, `3`, and so on.
    pub id: PostId,

    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,

    /// The current owner of a given post.
    pub owner: T::AccountId,

    /// Through post extension you can provide specific information necessary for different kinds 
    /// of posts such as regular posts, comments, and shared posts.
    pub extension: PostExtension,

    /// An id of a channel which contains a given post.
    pub channel_id: Option<ChannelId>,

    pub content: Content,

    /// Hidden field is used to recommend to end clients (web and mobile apps) that a particular 
    /// posts and its' comments should not be shown.
    pub hidden: bool,

    /// The total number of replies for a given post.
    pub replies_count: u16,

    /// The number of hidden replies for a given post.
    pub hidden_replies_count: u16,

    /// The number of times a given post has been shared.
    pub shares_count: u16,

    /// The number of times a given post has been upvoted.
    pub upvotes_count: u16,

    /// The number of times a given post has been downvoted.
    pub downvotes_count: u16,

    pub score: i32,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostUpdate {
    /// Deprecated: This field has no effect in `fn update_post()` extrinsic.
    /// See `fn move_post()` extrinsic if you want to move a post to another channel.
    pub channel_id: Option<ChannelId>,

    pub content: Option<Content>,
    pub hidden: Option<bool>,
}

/// Post extension provides specific information necessary for different kinds 
/// of posts such as regular posts, comments, and shared posts.
#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(untagged))]
pub enum PostExtension {
    RegularPost,
    Comment(Comment),
    SharedPost(PostId),
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Comment {
    pub parent_id: Option<PostId>,
    pub root_post_id: PostId,
}

impl Default for PostExtension {
    fn default() -> Self {
        PostExtension::RegularPost
    }
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + slixon_utils::Trait
    + slixon_channel_follows::Trait
    + slixon_channels::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Max comments depth
    type MaxCommentDepth: Get<u32>;

    type PostScores: PostScores<Self>;

    type AfterPostUpdated: AfterPostUpdated<Self>;

    type IsPostBlocked: IsPostBlocked<PostId>;
}

pub trait PostScores<T: Trait> {
    fn score_post_on_new_share(account: T::AccountId, original_post: &mut Post<T>) -> DispatchResult;
    fn score_root_post_on_new_comment(account: T::AccountId, root_post: &mut Post<T>) -> DispatchResult;
}

impl<T: Trait> PostScores<T> for () {
    fn score_post_on_new_share(_account: T::AccountId, _original_post: &mut Post<T>) -> DispatchResult {
        Ok(())
    }
    fn score_root_post_on_new_comment(_account: T::AccountId, _root_post: &mut Post<T>) -> DispatchResult {
        Ok(())
    }
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
pub trait AfterPostUpdated<T: Trait> {
    fn after_post_updated(account: T::AccountId, post: &Post<T>, old_data: PostUpdate);
}

pub const FIRST_POST_ID: u64 = 1;

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as PostsModule {

        /// The next post id.
        pub NextPostId get(fn next_post_id): PostId = FIRST_POST_ID;

        /// Get the details of a post by its' id.
        pub PostById get(fn post_by_id):
            map hasher(twox_64_concat) PostId => Option<Post<T>>;

        /// Get the ids of all direct replies by their parent's post id.
        pub ReplyIdsByPostId get(fn reply_ids_by_post_id):
            map hasher(twox_64_concat) PostId => Vec<PostId>;

        /// Get the ids of all posts in a given channel, by the channel's id.
        pub PostIdsByChannelId get(fn post_ids_by_channel_id):
            map hasher(twox_64_concat) ChannelId => Vec<PostId>;

        // TODO rename 'Shared...' to 'Sharing...'
        /// Get the ids of all posts that have shared a given original post id.
        pub SharedPostIdsByOriginalPostId get(fn shared_post_ids_by_original_post_id):
            map hasher(twox_64_concat) PostId => Vec<PostId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        PostCreated(AccountId, PostId),
        PostUpdated(AccountId, PostId),
        PostDeleted(AccountId, PostId),
        PostShared(AccountId, PostId),
        PostMoved(AccountId, PostId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {

        // Post related errors:

        /// Post was not found by id.
        PostNotFound,
        /// An account is not a post owner.
        NotAPostOwner,
        /// Nothing to update in this post.
        NoUpdatesForPost,
        /// Root post should have a channel id.
        PostHasNoChannelId,
        /// Not allowed to create a post/comment when a scope (channel or root post) is hidden.
        CannotCreateInHiddenScope,
        /// Post has no replies.
        NoRepliesOnPost,
        /// Cannot move a post to the same channel.
        CannotMoveToSameChannel,

        // Sharing related errors:

        /// Original post not found when sharing.
        OriginalPostNotFound,
        /// Cannot share a post that that is sharing another post.
        CannotShareSharingPost,
        /// This post's extension is not a `SharedPost`.
        NotASharingPost,

        // Comment related errors:

        /// Unknown parent comment id.
        UnknownParentComment,
        /// Post by `parent_id` is not of a `Comment` extension.
        NotACommentByParentId,
        /// Cannot update channel id of a comment.
        CannotUpdateChannelIdOnComment,
        /// Max comment depth reached.
        MaxCommentDepthReached,
        /// Only comment owner can update this comment.
        NotACommentAuthor,
        /// This post's extension is not a `Comment`.
        NotComment,

        // Permissions related errors:

        /// User has no permission to create root posts in this channel.
        NoPermissionToCreatePosts,
        /// User has no permission to create comments (aka replies) in this channel.
        NoPermissionToCreateComments,
        /// User has no permission to share posts/comments from this channel to another channel.
        NoPermissionToShare,
        /// User has no permission to update any posts in this channel.
        NoPermissionToUpdateAnyPost,
        /// A post owner is not allowed to update their own posts in this channel.
        NoPermissionToUpdateOwnPosts,
        /// A comment owner is not allowed to update their own comments in this channel.
        NoPermissionToUpdateOwnComments,
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    const MaxCommentDepth: u32 = T::MaxCommentDepth::get();

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 100_000 + T::DbWeight::get().reads_writes(8, 8)]
    pub fn create_post(
      origin,
      channel_id_opt: Option<ChannelId>,
      extension: PostExtension,
      content: Content
    ) -> DispatchResult {
      let creator = ensure_signed(origin)?;

      Utils::<T>::is_valid_content(content.clone())?;

      let new_post_id = Self::next_post_id();
      let new_post: Post<T> = Post::new(new_post_id, creator.clone(), channel_id_opt, extension, content.clone());

      // Get channel from either channel_id_opt or Comment if a comment provided
      let channel = &mut new_post.get_channel()?;
      ensure!(!channel.hidden, Error::<T>::CannotCreateInHiddenScope);

      ensure!(T::IsAccountBlocked::is_allowed_account(creator.clone(), channel.id), UtilsError::<T>::AccountIsBlocked);
      ensure!(T::IsContentBlocked::is_allowed_content(content, channel.id), UtilsError::<T>::ContentIsBlocked);

      let root_post = &mut new_post.get_root_post()?;
      ensure!(!root_post.hidden, Error::<T>::CannotCreateInHiddenScope);

      // Check whether account has permission to create Post (by extension)
      let mut permission_to_check = ChannelPermission::CreatePosts;
      let mut error_on_permission_failed = Error::<T>::NoPermissionToCreatePosts;

      if let PostExtension::Comment(_) = extension {
        permission_to_check = ChannelPermission::CreateComments;
        error_on_permission_failed = Error::<T>::NoPermissionToCreateComments;
      }

      Channels::ensure_account_has_channel_permission(
        creator.clone(),
        &channel,
        permission_to_check,
        error_on_permission_failed.into()
      )?;

      match extension {
        PostExtension::RegularPost => channel.inc_posts(),
        PostExtension::SharedPost(post_id) => Self::create_sharing_post(&creator, new_post_id, post_id, channel)?,
        PostExtension::Comment(comment_ext) => Self::create_comment(&creator, new_post_id, comment_ext, root_post)?,
      }

      if new_post.is_root_post() {
        ChannelById::insert(channel.id, channel.clone());
        PostIdsByChannelId::mutate(channel.id, |ids| ids.push(new_post_id));
      }

      PostById::insert(new_post_id, new_post);
      NextPostId::mutate(|n| { *n += 1; });

      Self::deposit_event(RawEvent::PostCreated(creator, new_post_id));
      Ok(())
    }

    #[weight = 100_000 + T::DbWeight::get().reads_writes(5, 3)]
    pub fn update_post(origin, post_id: PostId, update: PostUpdate) -> DispatchResult {
      let editor = ensure_signed(origin)?;

      let has_updates =
        update.content.is_some() ||
        update.hidden.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesForPost);

      let mut post = Self::require_post(post_id)?;
      let mut channel_opt = post.try_get_channel();

      if let Some(channel) = &channel_opt {
        ensure!(T::IsAccountBlocked::is_allowed_account(editor.clone(), channel.id), UtilsError::<T>::AccountIsBlocked);
        Self::ensure_account_can_update_post(&editor, &post, channel)?;
      }

      let mut is_update_applied = false;
      let mut old_data = PostUpdate::default();

      if let Some(content) = update.content {
        if content != post.content {
          Utils::<T>::is_valid_content(content.clone())?;

          if let Some(channel) = &channel_opt {
            ensure!(
              T::IsContentBlocked::is_allowed_content(content.clone(), channel.id),
              UtilsError::<T>::ContentIsBlocked
            );
          }

          old_data.content = Some(post.content.clone());
          post.content = content;
          is_update_applied = true;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != post.hidden {
          channel_opt = channel_opt.map(|mut channel| {
            if hidden {
              channel.inc_hidden_posts();
            } else {
              channel.dec_hidden_posts();
            }

            channel
          });

          if let PostExtension::Comment(comment_ext) = post.extension {
            Self::update_counters_on_comment_hidden_change(&comment_ext, hidden)?;
          }

          old_data.hidden = Some(post.hidden);
          post.hidden = hidden;
          is_update_applied = true;
        }
      }

      // Update this post only if at least one field should be updated:
      if is_update_applied {
        post.updated = Some(WhoAndWhen::<T>::new(editor.clone()));

        if let Some(channel) = channel_opt {
          <ChannelById<T>>::insert(channel.id, channel);
        }

        <PostById<T>>::insert(post.id, post.clone());
        T::AfterPostUpdated::after_post_updated(editor.clone(), &post, old_data);

        Self::deposit_event(RawEvent::PostUpdated(editor, post_id));
      }
      Ok(())
    }

    #[weight = T::DbWeight::get().reads(1) + 50_000]
    pub fn move_post(origin, post_id: PostId, new_channel_id: Option<ChannelId>) -> DispatchResult {
      let who = ensure_signed(origin)?;

      let post = &mut Self::require_post(post_id)?;

      ensure!(new_channel_id != post.channel_id, Error::<T>::CannotMoveToSameChannel);

      if let Some(channel) = post.try_get_channel() {
        Self::ensure_account_can_update_post(&who, &post, &channel)?;
      } else {
        post.ensure_owner(&who)?;
      }

      let old_channel_id = post.channel_id;

      if let Some(channel_id) = new_channel_id {
        Self::move_post_to_channel(who.clone(), post, channel_id)?;
      } else {
        Self::delete_post_from_channel(post_id)?;
      }

      let historical_data = PostUpdate {
        channel_id: old_channel_id,
        content: None,
        hidden: None,
      };

      T::AfterPostUpdated::after_post_updated(who.clone(), &post, historical_data);

      Self::deposit_event(RawEvent::PostMoved(who, post_id));
      Ok(())
    }
  }
}
