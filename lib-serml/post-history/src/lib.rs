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
use frame_support::{decl_module, decl_storage};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::Vec;
use frame_system::{self as system};

use slixon_posts::{Post, PostUpdate, AfterPostUpdated};
use slixon_utils::{WhoAndWhen, PostId};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostHistoryRecord<T: Trait> {
    pub edited: WhoAndWhen<T>,
    pub old_data: PostUpdate,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + slixon_utils::Trait
    + slixon_posts::Trait
{}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as PostHistoryModule {
        pub EditHistory get(fn edit_history):
            map hasher(twox_64_concat) PostId => Vec<PostHistoryRecord<T>>;
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}

impl<T: Trait> PostHistoryRecord<T> {
    fn new(updated_by: T::AccountId, old_data: PostUpdate) -> Self {
        PostHistoryRecord {
            edited: WhoAndWhen::<T>::new(updated_by),
            old_data
        }
    }
}

impl<T: Trait> AfterPostUpdated<T> for Module<T> {
    fn after_post_updated(sender: T::AccountId, post: &Post<T>, old_data: PostUpdate) {
        <EditHistory<T>>::mutate(post.id, |ids|
            ids.push(PostHistoryRecord::<T>::new(sender, old_data)));
    }
}
