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

use std::marker::PhantomData;

use log::warn;

use crate::{
    network::{data::Network, Data},
    Recipient,
};

/// A wrapper needed only because of type system theoretical constraints. Sadness.
pub struct NetworkWrapper<D: Data, DN: Network<D>> {
    inner: DN,
    _phantom: PhantomData<D>,
}

impl<D: Data, DN: Network<D>> From<DN> for NetworkWrapper<D, DN> {
    fn from(inner: DN) -> Self {
        NetworkWrapper {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<D: Data, DN: Network<D>> NetworkWrapper<D, DN> {
    fn send<R>(&self, data: D, recipient: R)
    where
        R: Into<Recipient>,
    {
        if let Err(e) = self.inner.send(data, recipient.into()) {
            warn!(target: "aleph-network", "Error '{:?}' while sending an AlephBFT message to the network.", e);
        }
    }

    async fn next_event(&mut self) -> Option<D> {
        self.inner.next().await
    }
}

#[async_trait::async_trait]
impl<D: Data, DN: Network<D>> current_aleph_bft::Network<D> for NetworkWrapper<D, DN> {
    fn send(&self, data: D, recipient: current_aleph_bft::Recipient) {
        NetworkWrapper::send(self, data, recipient)
    }

    async fn next_event(&mut self) -> Option<D> {
        NetworkWrapper::next_event(self).await
    }
}

#[async_trait::async_trait]
impl<D: Data, DN: Network<D>> legacy_aleph_bft::Network<D> for NetworkWrapper<D, DN> {
    fn send(&self, data: D, recipient: legacy_aleph_bft::Recipient) {
        NetworkWrapper::send(self, data, recipient)
    }

    async fn next_event(&mut self) -> Option<D> {
        NetworkWrapper::next_event(self).await
    }
}
