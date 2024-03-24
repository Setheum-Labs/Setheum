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
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use parking_lot::Mutex;
use sp_consensus::SyncOracle as SyncOracleT;

const OFFLINE_THRESHOLD: Duration = Duration::from_secs(6);
const FAR_BEHIND_THRESHOLD: u32 = 15;
const MAJOR_SYNC_THRESHOLD: Duration = Duration::from_secs(10);

/// A sync oracle implementation tracking how recently the node was far behind the highest known justification.
/// It defines being in major sync as being more than 15 blocks behind the highest known justification less than 10 seconds ago.
/// It defines being offline as not getting any update for at least 6 seconds (or never at all).
#[derive(Clone)]
pub struct SyncOracle {
    last_far_behind: Arc<Mutex<Instant>>,
    last_update: Arc<Mutex<Instant>>,
    // TODO: remove when SyncingService is no longer needed
    is_major_syncing: Arc<AtomicBool>,
}

impl SyncOracle {
    pub fn new() -> (Self, Arc<AtomicBool>) {
        let is_major_syncing = Arc::new(AtomicBool::new(true));
        let oracle = SyncOracle {
            last_update: Arc::new(Mutex::new(Instant::now() - OFFLINE_THRESHOLD)),
            last_far_behind: Arc::new(Mutex::new(Instant::now())),
            is_major_syncing: is_major_syncing.clone(),
        };
        (oracle, is_major_syncing)
    }

    pub fn update_behind(&self, behind: u32) {
        let now = Instant::now();
        *self.last_update.lock() = now;
        if behind > FAR_BEHIND_THRESHOLD {
            *self.last_far_behind.lock() = now;
        }
        self.major_sync();
    }

    pub fn major_sync(&self) -> bool {
        let is_major_syncing = self.last_far_behind.lock().elapsed() < MAJOR_SYNC_THRESHOLD;
        self.is_major_syncing
            .store(is_major_syncing, Ordering::Relaxed);
        is_major_syncing
    }
}

impl Default for SyncOracle {
    fn default() -> Self {
        SyncOracle::new().0
    }
}

impl SyncOracleT for SyncOracle {
    fn is_major_syncing(&self) -> bool {
        self.major_sync()
    }

    fn is_offline(&self) -> bool {
        self.last_update.lock().elapsed() > OFFLINE_THRESHOLD
    }
}
