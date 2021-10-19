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

use sp_std::prelude::*;

use crate::{Module, Trait};

impl<T: Trait> Module<T> {
    pub fn filter_followed_accounts(account: T::AccountId, maybe_following: Vec<T::AccountId>) -> Vec<T::AccountId> {
        maybe_following.iter()
            .filter(|maybe_following| Self::account_followed_by_account((&account, maybe_following)))
            .cloned().collect()
    }
}