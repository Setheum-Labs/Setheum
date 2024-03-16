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

//! Synchronize with [Aleph Rate Limiter Clique](https://github.com/Cardinal-Cryptography/aleph-node/tree/main/rate-limiter)
//! This module Allows to limit access to some resource. Given a preferred rate (units of something) and
//! last used amount of units of some resource, it calculates how long we should delay our next access to
//! that resource in order to satisfy that rate.
mod rate_limiter;
mod token_bucket;

pub use crate::rate_limiter::{RateLimiter, SleepingRateLimiter};

const LOG_TARGET: &str = "rate-limiter";
