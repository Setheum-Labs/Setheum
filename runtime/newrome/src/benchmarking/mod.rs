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

#![cfg(feature = "runtime-benchmarks")]

// module benchmarking (SERML)
pub mod serp_auction;
pub mod serp_treasury;
pub mod dex;
pub mod settway;
pub mod incentives;
pub mod prices;
pub mod transaction_payment;

// orml benchmarking
pub mod auction;
pub mod authority;
pub mod currencies;
pub mod oracle;
pub mod tokens;
pub mod utils;
pub mod vesting;
