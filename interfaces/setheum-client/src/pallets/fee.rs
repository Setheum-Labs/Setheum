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

use crate::{
    api,
    sp_runtime::{traits::One, FixedU128},
    BlockHash, ConnectionApi,
};

/// Transaction payment pallet API.
#[async_trait::async_trait]
pub trait TransactionPaymentApi {
    /// API for [`next_fee_multiplier`](https://paritytech.github.io/substrate/master/pallet_transaction_payment/pallet/struct.Pallet.html#method.next_fee_multiplier) call.
    async fn get_next_fee_multiplier(&self, at: Option<BlockHash>) -> FixedU128;
}

#[async_trait::async_trait]
impl<C: ConnectionApi> TransactionPaymentApi for C {
    async fn get_next_fee_multiplier(&self, at: Option<BlockHash>) -> FixedU128 {
        let addrs = api::storage().transaction_payment().next_fee_multiplier();

        self.get_storage_entry_maybe(&addrs, at)
            .await
            .map_or(FixedU128::one(), |f| FixedU128::from_inner(f.0))
    }
}
