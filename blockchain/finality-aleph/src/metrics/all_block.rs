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

use log::warn;
use substrate_prometheus_endpoint::Registry;

use super::{finality_rate::FinalityRateMetrics, timing::DefaultClock, Checkpoint};
use crate::{metrics::LOG_TARGET, BlockId, TimingBlockMetrics};

/// Wrapper around various block-related metrics.
#[derive(Clone)]
pub struct AllBlockMetrics {
    timing_metrics: TimingBlockMetrics<DefaultClock>,
    finality_rate_metrics: FinalityRateMetrics,
}

impl AllBlockMetrics {
    pub fn new(registry: Option<&Registry>) -> Self {
        let timing_metrics = match TimingBlockMetrics::new(registry, DefaultClock) {
            Ok(timing_metrics) => timing_metrics,
            Err(e) => {
                warn!(
                    target: LOG_TARGET,
                    "Failed to register Prometheus block timing metrics: {:?}.", e
                );
                TimingBlockMetrics::Noop
            }
        };
        let finality_rate_metrics = match FinalityRateMetrics::new(registry) {
            Ok(finality_rate_metrics) => finality_rate_metrics,
            Err(e) => {
                warn!(
                    target: LOG_TARGET,
                    "Failed to register Prometheus finality rate metrics: {:?}.", e
                );
                FinalityRateMetrics::Noop
            }
        };
        AllBlockMetrics {
            timing_metrics,
            finality_rate_metrics,
        }
    }

    /// Triggers all contained block metrics.
    pub fn report_block(&self, block_id: BlockId, checkpoint: Checkpoint, own: Option<bool>) {
        self.timing_metrics
            .report_block(block_id.hash(), checkpoint);
        self.finality_rate_metrics.report_block(
            block_id.hash(),
            block_id.number(),
            checkpoint,
            own,
        );
    }
}
