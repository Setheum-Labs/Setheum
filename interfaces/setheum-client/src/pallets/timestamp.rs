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

use crate::{api, BlockHash, ConnectionApi};

/// Timestamp payment pallet API.
#[async_trait::async_trait]
pub trait TimestampApi {
    /// API for [`get`](https://paritytech.github.io/substrate/master/pallet_timestamp/pallet/struct.Pallet.html#method.get) call.
    async fn get_timestamp(&self, at: Option<BlockHash>) -> Option<u64>;
}

#[async_trait::async_trait]
impl<C: ConnectionApi> TimestampApi for C {
    async fn get_timestamp(&self, at: Option<BlockHash>) -> Option<u64> {
        let addrs = api::storage().timestamp().now();
        self.get_storage_entry_maybe(&addrs, at).await
    }
}
