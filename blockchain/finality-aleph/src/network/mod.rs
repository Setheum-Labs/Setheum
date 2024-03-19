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

use parity_scale_codec::Codec;

pub mod address_cache;
pub mod data;
mod gossip;
#[cfg(test)]
pub mod mock;
pub mod session;
mod substrate;
pub mod tcp;

#[cfg(test)]
pub use gossip::mock::{MockEvent, MockRawNetwork};
pub use gossip::{
    Error as GossipError, Network as GossipNetwork, Protocol, Service as GossipService,
};
use network_clique::{AddressingInformation, NetworkIdentity, PeerId};
pub use substrate::{
    NetworkEventStream as SubstrateNetworkEventStream, ProtocolNaming, SubstrateNetwork,
};

/// A basic alias for properties we expect basic data to satisfy.
pub trait Data: Clone + Codec + Send + Sync + 'static {}

impl<D: Clone + Codec + Send + Sync + 'static> Data for D {}
