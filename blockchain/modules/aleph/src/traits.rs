// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of Setheum.

// Copyright (C) 2019-Present Setheum Labs.
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

use frame_support::sp_runtime::{traits::OpaqueKeys, RuntimeAppPublic};
use primitives::AuthorityId;
use sp_std::prelude::*;

use crate::Config;

/// Authorities provider, used only as default value in case of missing this information in our pallet. This can
/// happen for the session after runtime upgraded.
pub trait NextSessionAuthorityProvider<T: Config> {
    fn next_authorities() -> Vec<T::AuthorityId>;
}

impl<T> NextSessionAuthorityProvider<T> for pallet_session::Pallet<T>
where
    T: Config + pallet_session::Config,
{
    fn next_authorities() -> Vec<T::AuthorityId> {
        let next: Option<Vec<_>> = pallet_session::Pallet::<T>::queued_keys()
            .iter()
            .map(|(_, key)| key.get(AuthorityId::ID))
            .collect();

        next.unwrap_or_else(|| {
            log::error!(target: "module_aleph", "Missing next session keys");
            vec![]
        })
    }
}
