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

mod all_block;
mod chain_state;
mod finality_rate;
mod timing;
pub mod transaction_pool;

pub use all_block::AllBlockMetrics;
pub use chain_state::run_chain_state_metrics;
pub use finality_rate::FinalityRateMetrics;
use substrate_prometheus_endpoint::{exponential_buckets, prometheus};
pub use timing::{Checkpoint, DefaultClock, TimingBlockMetrics};
pub use transaction_pool::TransactionPoolInfoProvider;

const LOG_TARGET: &str = "aleph-metrics";

/// Create `count_below` + 1 + `count_above` buckets, where (`count_below` + 1)th bucket
/// has an upper bound `start`. The buckets are exponentially distributed with a factor `factor`.
pub fn exponential_buckets_two_sided(
    start: f64,
    factor: f64,
    count_below: usize,
    count_above: usize,
) -> prometheus::Result<Vec<f64>> {
    let mut strictly_smaller =
        exponential_buckets(start / factor.powi(count_below as i32), factor, count_below)?;
    let mut greater_than_or_equal = exponential_buckets(start, factor, 1 + count_above)?;
    if let Some(last_smaller) = strictly_smaller.last() {
        if last_smaller >= &start {
            return Err(prometheus::Error::Msg(
                "Floating point arithmetic error causing incorrect buckets, try larger factor or smaller count_below"
                    .to_string(),
            ));
        }
    }
    strictly_smaller.append(&mut greater_than_or_equal);
    Ok(strictly_smaller)
}
