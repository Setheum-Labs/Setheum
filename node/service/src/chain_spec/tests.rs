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

//! Unit tests for the genesis resources data.

#![cfg(test)]

use setheum_primitives::{AccountId, Balance, BlockNumber};

#[test]
#[cfg(feature = "with-setheum-runtime")]
fn setheum_foundation_accounts_config_is_correct() {
	use sp_core::crypto::Ss58Codec;

	let setheum_foundation_accounts = setheum_runtime::SetheumFoundationAccounts::get();
	assert!(setheum_foundation_accounts
		.contains(&AccountId::from_string("5DhvNsZdYTtWUYdHvREWhsHWt1StP9bA21vsC1Wp6UksjNAh").unwrap()),);
	// Todo: Update vvvvvvvvvvvvv!
	// assert!(setheum_foundation_accounts
	// 	.contains(&AccountId::from_string("pndshZqDAC9GutDvv7LzhGhgWeGv5YX9puFA8xDidHXCyjd").unwrap()),);
}

#[test]
fn check_setheum_allocation() {
	let allocation_json = &include_bytes!("../../../../resources/setheum-allocation-DNAR.json")[..];
	let _: Vec<(AccountId, Balance)> = serde_json::from_slice(allocation_json).unwrap();
}

#[test]
fn check_setheum_airdrop() {
	let airdrop_json = &include_bytes!("../../../../resources/newrome-airdrop-DNAR.json")[..];
	let _: Vec<(AccountId, Balance)> = serde_json::from_slice(airdrop_json).unwrap();
}

#[test]
fn check_nfts() {
	let nfts_json = &include_bytes!("../../../../resources/newrome-airdrop-NFT.json")[..];
	let _: Vec<(
		AccountId,
		Vec<u8>,
		module_nft::ClassData<Balance>,
		Vec<(Vec<u8>, module_nft::TokenData<Balance>, Vec<AccountId>)>,
	)> = serde_json::from_slice(nfts_json).unwrap();
}
