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

use std::{
    fmt::{Debug, Display},
    hash::Hash,
    marker::Send,
};

use crate::{
    block::{Justification, UnverifiedHeader},
    BlockId,
};

mod data;
mod forest;
mod handler;
mod message_limiter;
mod metrics;
mod service;
mod task_queue;
mod tasks;
mod ticker;

pub use data::MAX_MESSAGE_SIZE;
pub use handler::DatabaseIO;
pub use service::{Service, IO};

const LOG_TARGET: &str = "aleph-block-sync";

/// The identifier of a connected peer.
pub trait PeerId: Debug + Clone + Hash + Eq {}

impl<T: Debug + Clone + Hash + Eq> PeerId for T {}

/// An interface for submitting additional justifications to the justification sync.
/// Chiefly ones created by ABFT, but others will also be handled appropriately.
/// The block corresponding to the submitted `Justification` MUST be obtained and
/// imported into the Substrate database by the user, as soon as possible.
pub trait JustificationSubmissions<J: Justification>: Clone + Send + 'static {
    type Error: Display;

    /// Submit a justification to the underlying justification sync.
    fn submit(&mut self, justification: J::Unverified) -> Result<(), Self::Error>;
}

/// An interface for requesting specific blocks from the block sync.
/// Required by the data availability mechanism in ABFT.
pub trait RequestBlocks<UH: UnverifiedHeader>: Clone + Send + Sync + 'static {
    type Error: Display;

    /// Request the given block.
    fn request_block(&self, header: UH) -> Result<(), Self::Error>;
}

/// An interface for requesting specific blocks from the block sync.
/// Required by the data availability mechanism in ABFT.
// TODO: Remove this after support for headerless proposals gets dropped.
pub trait LegacyRequestBlocks: Clone + Send + Sync + 'static {
    type Error: Display;

    /// Request the given block.
    fn request_block(&self, block_id: BlockId) -> Result<(), Self::Error>;
}

#[cfg(test)]
pub type MockPeerId = u32;
