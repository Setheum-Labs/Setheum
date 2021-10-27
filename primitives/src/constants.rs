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

/// Money matters.
pub mod currency {
    use crate::Balance;

	// another implementation is in `runtime_common`
	pub const fn dollar(currency_id: CurrencyId) -> Balance {
		10u128.saturating_pow(currency_id.decimals().expect("Not support Erc20 decimals").into())
	}

	pub const fn cent(currency_id: CurrencyId) -> Balance {
		dollar(currency_id) / 100
	}

	pub const fn millicent(currency_id: CurrencyId) -> Balance {
		cent(currency_id) / 1000
	}

	pub const fn microcent(currency_id: CurrencyId) -> Balance {
		millicent(currency_id) / 1000
	}
	
	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 2 * dollar(SETM) + (bytes as Balance) * 10 * millicent(SETM)
	}
    // GPoS rewards in the first year
    pub const FIRST_YEAR_REWARDS: Balance = 5_000_000 * dollar(SETM);

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 1_000 * CENTS + (bytes as Balance) * 100 * MILLICENTS
	}
}

/// Time and blocks.
pub mod time {
    use crate::{BlockNumber, Moment};

	// 3 seconds average blocktime
	pub const SECS_PER_BLOCK: Moment = 3;
	pub const MILLISECS_PER_BLOCK: Moment = SECS_PER_BLOCK * 1000;

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

    // Use different settings in the test
    #[cfg(feature = "test")]
    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
    #[cfg(not(feature = "test"))]
    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 1 * HOURS;
    
    // Use different settings in the test
    #[cfg(feature = "test")]
	pub const EPOCH_DURATION_IN_SLOTS: u64 = {
		const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

		(EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
	};
    #[cfg(not(feature = "test"))]
	pub const EPOCH_DURATION_IN_SLOTS: u64 = {
		const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

		(EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
	};
}

pub mod staking {
    use crate::Balance;
    // The reward decrease ratio per year
    pub const REWARD_DECREASE_RATIO: (Balance, Balance) = (88, 100);
    // The minimal reward ratio
    pub const MIN_REWARD_RATIO: (Balance, Balance) = (28, 1000);
    // The start year for extra reward
    pub const EXTRA_REWARD_START_YEAR: u64 = 4;
}

pub mod swork {
    use super::time::*;

    // Use different settings in the test
    #[cfg(feature = "test")]
    pub const REPORT_SLOT: u64 = EPOCH_DURATION_IN_BLOCKS as u64 * 3;
    #[cfg(not(feature = "test"))]
    pub const REPORT_SLOT: u64 = EPOCH_DURATION_IN_BLOCKS as u64;

    pub const UPDATE_OFFSET: u32 = (REPORT_SLOT / 3) as u32;
    pub const END_OFFSET: u32 = 1;
}

pub mod market {
    pub const BASE_FEE_UPDATE_SLOT: u32 = 600;
    pub const BASE_FEE_UPDATE_OFFSET: u32 = 22;

    pub const PRICE_UPDATE_SLOT: u32 = 10;
    pub const PRICE_UPDATE_OFFSET: u32 = 3;
    pub const FILES_COUNT_REFERENCE: u32 = 20_000_000; // 20_000_000 / 50_000_000 = 40%

    pub const SPOWER_UPDATE_SLOT: u32 = 100;
    pub const SPOWER_UPDATE_OFFSET: u32 = 7;
    pub const MAX_PENDING_FILES: usize = 20;


    // Use different settings in the test
    #[cfg(feature = "test")]
    pub const COLLATERAL_RATIO: u32 = 10;
    #[cfg(not(feature = "test"))]
    pub const COLLATERAL_RATIO: u32 = 1;
}
