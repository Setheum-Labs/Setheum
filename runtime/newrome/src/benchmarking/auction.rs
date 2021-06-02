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
	dollar, Auction, AuctionId, SerpAuction, AuctionTimeToClose, SerpTreasury, Runtime, System, ROME, rUSD, rSETT,
};

use super::utils::set_balance;
use frame_benchmarking::account;
use frame_support::traits::OnFinalize;
use frame_system::RawOrigin;
use setheum_support::{SerpAuction as SerpAuctionTrait, SerpTreasury};
use orml_benchmarking::runtime_benchmarks;
use sp_std::prelude::*;

const SEED: u32 = 0;
const MAX_DOLLARS: u32 = 1000;
const MAX_AUCTION_ID: u32 = 100;

runtime_benchmarks! {
	{ Runtime, orml_auction }

	_ {
		let d in 1 .. MAX_DOLLARS => ();
		let c in 1 .. MAX_AUCTION_ID => ();
	}

	// `bid` a setter auction, best cases:
	// there's no bidder before and bid price doesn't exceed target amount
	#[extra]
	bid_setter_auction_as_first_bidder {
		let bidder = account("bidder", 0, SEED);
		let funder = account("funder", 0, SEED);
		let currency_id = rSETT;
		let reserve_amount = 100 * dollar(currency_id);
		let target_amount = 10_000 * dollar(rUSD);
		let bid_price = (5_000u128 + d as u128) * dollar(rUSD);
		let auction_id: AuctionId = 0;

		set_balance(currency_id, &funder, reserve_amount);
		set_balance(rUSD, &bidder, bid_price);
		<SerpTreasury as SerpTreasury<_>>::deposit_reserve(&funder, currency_id, reserve_amount)?;
		SerpAuction::new_setter_auction(&funder, currency_id, reserve_amount, target_amount)?;
	}: bid(RawOrigin::Signed(bidder), auction_id, bid_price)

	// `bid` a setter auction, worst cases:
	// there's bidder before and bid price will exceed target amount
	bid_setter_auction {
		let bidder = account("bidder", 0, SEED);
		let previous_bidder = account("previous_bidder", 0, SEED);
		let funder = account("funder", 0, SEED);
		let currency_id = rSETT;
		let reserve_amount = 100 * dollar(currency_id);
		let target_amount = 10_000 * dollar(rUSD);
		let previous_bid_price = (5_000u128 + d as u128) * dollar(rUSD);
		let bid_price = (10_000u128 + d as u128) * dollar(rUSD);
		let auction_id: AuctionId = 0;

		set_balance(currency_id, &funder, reserve_amount);
		set_balance(rUSD, &bidder, bid_price);
		set_balance(rUSD, &previous_bidder, previous_bid_price);
		<SerpTreasury as SerpTreasury<_>>::deposit_reserve(&funder, currency_id, reserve_amount)?;
		SerpAuction::new_setter_auction(&funder, currency_id, reserve_amount, target_amount)?;
		Auction::bid(RawOrigin::Signed(previous_bidder).into(), auction_id, previous_bid_price)?;
	}: bid(RawOrigin::Signed(bidder), auction_id, bid_price)

	// `bid` a serplus auction, best cases:
	// there's no bidder before
	#[extra]
	bid_serplus_auction_as_first_bidder {
		let bidder = account("bidder", 0, SEED);

		let serplusamount = 100 * dollar(rUSD);
		let bid_price = d * dollar(ROME);
		let auction_id: AuctionId = 0;

		set_balance(ROME, &bidder, bid_price);
		SerpAuction::new_serplus_auction(serplusamount)?;
	}: bid(RawOrigin::Signed(bidder), auction_id, bid_price)

	// `bid` a serplus auction, worst cases:
	// there's bidder before
	bid_serplus_auction {
		let bidder = account("bidder", 0, SEED);
		let previous_bidder = account("previous_bidder", 0, SEED);
		let serplusamount = 100 * dollar(rUSD);
		let bid_price = (d as u128 * 2u128) * dollar(ROME);
		let previous_bid_price = d * dollar(ROME);
		let auction_id: AuctionId = 0;

		set_balance(ROME, &bidder, bid_price);
		set_balance(ROME, &previous_bidder, previous_bid_price);
		SerpAuction::new_serplus_auction(serplusamount)?;
		Auction::bid(RawOrigin::Signed(previous_bidder).into(), auction_id, previous_bid_price)?;
	}: bid(RawOrigin::Signed(bidder), auction_id, bid_price)

	// `bid` a diamond auction, best cases:
	// there's no bidder before and bid price happens to be standard amount
	#[extra]
	bid_diamond_auction_as_first_bidder {
		let bidder = account("bidder", 0, SEED);

		let fix_standard_amount = 100 * dollar(rUSD);
		let initial_amount = 10 * dollar(ROME);
		let auction_id: AuctionId = 0;

		set_balance(rUSD, &bidder, fix_standard_amount);
		SerpAuction::new_diamond_auction(initial_amount ,fix_standard_amount)?;
	}: bid(RawOrigin::Signed(bidder), auction_id, fix_standard_amount)

	// `bid` a diamond auction, worst cases:
	// there's bidder before
	bid_diamond_auction {
		let bidder = account("bidder", 0, SEED);
		let previous_bidder = account("previous_bidder", 0, SEED);
		let fix_standard_amount = 100 * dollar(rUSD);
		let initial_amount = 10 * dollar(ROME);
		let previous_bid_price = fix_standard_amount;
		let bid_price = fix_standard_amount * 2;
		let auction_id: AuctionId = 0;

		set_balance(rUSD, &bidder, bid_price);
		set_balance(rUSD, &previous_bidder, previous_bid_price);
		SerpAuction::new_diamond_auction(initial_amount ,fix_standard_amount)?;
		Auction::bid(RawOrigin::Signed(previous_bidder).into(), auction_id, previous_bid_price)?;
	}: bid(RawOrigin::Signed(bidder), auction_id, bid_price)

	on_finalize {
		let c in ...;

		let bidder = account("bidder", 0, SEED);
		let fix_standard_amount = 100 * dollar(rUSD);
		let initial_amount = 10 * dollar(ROME);
		let auction_id: AuctionId = 0;
		set_balance(rUSD, &bidder, fix_standard_amount * c as u128);

		System::set_block_number(1);
		for auction_id in 0 .. c {
			SerpAuction::new_diamond_auction(initial_amount ,fix_standard_amount)?;
			Auction::bid(RawOrigin::Signed(bidder.clone()).into(), auction_id, fix_standard_amount)?;
		}
	}: {
		Auction::on_finalize(System::block_number() + AuctionTimeToClose::get());
	}
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
	fn bid_setter_auction_as_first_bidder() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_bid_setter_auction_as_first_bidder());
		});
	}

	#[test]
	fn bid_setter_auction() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_bid_setter_auction());
		});
	}

	#[test]
	fn bid_serplus_auction_as_first_bidder() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_bid_serplus_auction_as_first_bidder());
		});
	}

	#[test]
	fn bid_serplus_auction() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_bid_serplus_auction());
		});
	}

	#[test]
	fn bid_diamond_auction_as_first_bidder() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_bid_diamond_auction_as_first_bidder());
		});
	}

	#[test]
	fn bid_diamond_auction() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_bid_diamond_auction());
		});
	}

	#[test]
	fn on_finalize() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_on_finalize());
		});
	}
}
