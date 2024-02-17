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

use crate::{ExchangeRate, Rate};
use sp_runtime::DispatchResult;
use xcm::v3::prelude::*;

pub trait LiquidStakingManager<AccountId, Balance> {
	/// Mint liquid currency by locking up staking currency
	fn mint(who: AccountId, amount: Balance) -> DispatchResult;
	/// Request for protocol to redeem liquid currency for staking currency
	fn request_redeem(who: AccountId, amount: Balance, fast_match: bool) -> DispatchResult;
	/// Calculates current exchange rate between staking and liquid currencies (staking : liquid)
	fn get_exchange_rate() -> ExchangeRate;
	/// Estimated return rate per era from liquid staking
	fn get_estimated_reward_rate() -> Rate;
	/// Gets commission rate of the Liquid Staking protocol
	fn get_commission_rate() -> Rate;
	/// Fee for fast matching redeem request
	fn get_fast_match_fee() -> Rate;
}
