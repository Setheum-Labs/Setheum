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

use codec::Codec;
use sp_std::vec::Vec;
use sp_std::collections::btree_map::BTreeMap;

use slixon_reactions::{
    ReactionId,
    ReactionKind,
    rpc::FlatReaction,
};
use slixon_utils::PostId;

sp_api::decl_runtime_apis! {
    pub trait ReactionsApi<AccountId, BlockNumber> where
        AccountId: Codec,
        BlockNumber: Codec
    {
        fn get_reactions_by_ids(reaction_ids: Vec<ReactionId>) -> Vec<FlatReaction<AccountId, BlockNumber>>;

        fn get_reactions_by_post_id(
            post_id: PostId,
            limit: u64,
            offset: u64
        ) -> Vec<FlatReaction<AccountId, BlockNumber>>;

        fn get_reaction_kinds_by_post_ids_and_reactor(
            post_ids: Vec<PostId>,
            reactor: AccountId,
        ) -> BTreeMap<PostId, ReactionKind>;
    }
}
