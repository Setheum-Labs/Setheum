// This file is part of Setheum.

// Copyright (C) 2020-2021 Setheum Labs.
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

use crate::{
	dollar, SetheumOracle, AccountId, AuctionId, AuctionManager, SerpTreasury, Currencies, EmergencyShutdown,
	GetNativeCurrencyId, GetStableCurrencyId, Price, Runtime, DOT,
};

use super::utils::set_balance;
use frame_benchmarking::account;
use frame_system::RawOrigin;
use setheum_support::AuctionManager as AuctionManagerTrait;
use setheum_support::SerpTreasury;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::MultiCurrency;
use sp_runtime::FixedPointNumber;
use sp_std::prelude::*;

const SEED: u32 = 0;

runtime_benchmarks! {
	{ Runtime, setheum_auction_manager }

	_ {}

	// `cancel` a surplus auction, worst case:
	// auction have been already bid
	cancel_surplus_auction {
		let bidder: AccountId = account("bidder", 0, SEED);
		let native_currency_id = GetNativeCurrencyId::get();
		let stable_currency_id = GetStableCurrencyId::get();

		// set balance
		set_balance(native_currency_id, &bidder, 10 * dollar(native_currency_id));

		// create surplus auction
		<AuctionManager as AuctionManagerTrait<AccountId>>::new_surplus_auction(dollar(stable_currency_id))?;
		let auction_id: AuctionId = Default::default();

		// bid surplus auction
		let _ = AuctionManager::surplus_auction_bid_handler(1, auction_id, (bidder, dollar(native_currency_id)), None);

		// shutdown
		EmergencyShutdown::emergency_shutdown(RawOrigin::Root.into())?;
	}: cancel(RawOrigin::None, auction_id)

	// `cancel` a diamond auction, worst case:
	// auction have been already bid
	cancel_diamond_auction {
		let bidder: AccountId = account("bidder", 0, SEED);
		let native_currency_id = GetNativeCurrencyId::get();
		let stable_currency_id = GetStableCurrencyId::get();

		// set balance
		set_balance(stable_currency_id, &bidder, 10 * dollar(stable_currency_id));

		// create diamond auction
		<AuctionManager as AuctionManagerTrait<AccountId>>::new_diamond_auction(dollar(native_currency_id), 10 * dollar(stable_currency_id))?;
		let auction_id: AuctionId = Default::default();

		// bid diamond auction
		let _ = AuctionManager::diamond_auction_bid_handler(1, auction_id, (bidder, 20 * dollar(stable_currency_id)), None);

		// shutdown
		EmergencyShutdown::emergency_shutdown(RawOrigin::Root.into())?;
	}: cancel(RawOrigin::None, auction_id)

	// `cancel` a setter auction, worst case:
	// auction have been already bid
	cancel_setter_auction {
		let bidder: AccountId = account("bidder", 0, SEED);
		let funder: AccountId = account("funder", 0, SEED);
		let stable_currency_id = GetStableCurrencyId::get();

		// set balance
		Currencies::deposit(stable_currency_id, &bidder, 80 * dollar(stable_currency_id))?;
		Currencies::deposit(DOT, &funder, dollar(DOT))?;
		SerpTreasury::deposit_reserve(&funder, DOT, dollar(DOT))?;

		// feed price
		SetheumOracle::feed_values(RawOrigin::Root.into(), vec![(DOT, Price::saturating_from_integer(120))])?;

		// create setter auction
		AuctionManager::new_setter_auction(&funder, DOT, dollar(DOT), 100 * dollar(stable_currency_id))?;
		let auction_id: AuctionId = Default::default();

		// bid setter auction
		let _ = AuctionManager::setter_auction_bid_handler(1, auction_id, (bidder, 80 * dollar(stable_currency_id)), None);

		// shutdown
		EmergencyShutdown::emergency_shutdown(RawOrigin::Root.into())?;
	}: cancel(RawOrigin::None, auction_id)
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::assert_ok;

	fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap()
			.into()
	}

	#[test]
	fn test_cancel_surplus_auction() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_cancel_surplus_auction());
		});
	}

	#[test]
	fn test_cancel_diamond_auction() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_cancel_diamond_auction());
		});
	}

	#[test]
	fn test_cancel_setter_auction() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_cancel_setter_auction());
		});
	}
}
