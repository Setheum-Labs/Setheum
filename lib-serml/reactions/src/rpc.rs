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

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Zero;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;

use slixon_utils::{PostId, rpc::FlatWhoAndWhen};

use crate::{Module, Reaction, ReactionId, ReactionKind, Trait};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatReaction<AccountId, BlockNumber> {
    pub id: ReactionId,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub who_and_when: FlatWhoAndWhen<AccountId, BlockNumber>,
    pub kind: ReactionKind,
}

#[cfg(feature = "std")]
impl Serialize for ReactionKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let reaction_kind_bytes: &[u8; 1] = match self {
            ReactionKind::Upvote => b"U",
            ReactionKind::Downvote => b"D"
        };

        serializer.serialize_str(
            std::str::from_utf8(reaction_kind_bytes).unwrap_or_default()
        )
    }
}

impl<T: Trait> From<Reaction<T>> for FlatReaction<T::AccountId, T::BlockNumber> {
    fn from(from: Reaction<T>) -> Self {
        let Reaction { id, created, updated, kind } = from;

        Self {
            id,
            who_and_when: (created, updated).into(),
            kind,
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_reactions_by_ids(
        reaction_ids: Vec<ReactionId>
    ) -> Vec<FlatReaction<T::AccountId, T::BlockNumber>> {
        reaction_ids.iter()
                    .filter_map(|id| Self::require_reaction(*id).ok())
                    .map(|reaction| reaction.into())
                    .collect()
    }

    pub fn get_reactions_by_post_id(
        post_id: PostId,
        limit: u64,
        offset: u64,
    ) -> Vec<FlatReaction<T::AccountId, T::BlockNumber>> {
        let mut reactions = Vec::new();

        let reaction_ids: Vec<PostId> = Self::reaction_ids_by_post_id(&post_id);
        let mut i = reaction_ids.len().saturating_sub(1 + offset as usize);

        while reactions.len() < limit as usize {
            if let Some(reaction_id) = reaction_ids.get(i) {
                if let Ok(reaction) = Self::require_reaction(*reaction_id) {
                    reactions.push(reaction.into());
                }
            }

            if i.is_zero() { break; }

            i = i.saturating_sub(1);
        }

        reactions
    }

    pub fn get_reaction_kinds_by_post_ids_and_reactor(
        post_ids: Vec<PostId>,
        reactor: T::AccountId,
    ) -> BTreeMap<PostId, ReactionKind> {
        let res = post_ids.iter()
            .filter_map(|post_id| Some(*post_id).zip(
                Option::from(Self::post_reaction_id_by_account((&reactor, post_id)))
                    .filter(|v| *v != 0)
                    .and_then(|reaction_id|
                        Self::require_reaction(reaction_id).ok().map(|reaction| reaction.kind)
                    )
            ));

        res.clone().collect()
    }
}