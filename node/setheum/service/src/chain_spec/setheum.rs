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

use setheum_primitives::{AccountId, TokenSymbol};
use hex_literal::hex;
use sc_chain_spec::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde_json::map::Map;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::crypto::UncheckedInto;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{FixedPointNumber, FixedU128};

use crate::chain_spec::{Extensions, TELEMETRY_URL};

pub type ChainSpec = sc_service::GenericChainSpec<setheum_runtime::GenesisConfig, Extensions>;

pub fn setheum_config() -> Result<ChainSpec, String> {
	Err("Not available".into())
}

pub fn latest_setheum_config() -> Result<ChainSpec, String> {
	let mut properties = Map::new();
	let mut token_symbol: Vec<String> = vec![];
	let mut token_decimals: Vec<u32> = vec![];
	TokenSymbol::get_info().iter().for_each(|(symbol_name, decimals)| {
		token_symbol.push(symbol_name.to_string());
		token_decimals.push(*decimals);
	});
	properties.insert("tokenSymbol".into(), token_symbol.into());
	properties.insert("tokenDecimals".into(), token_decimals.into());

	let wasm_binary = setheum_runtime::WASM_BINARY.ok_or("setheum runtime wasm binary not available")?;

	Ok(ChainSpec::from_genesis(
		"setheum Mainnet",
		"setheum",
		ChainType::Live,
		//
		// SECRET="..."
		//
		// ROOT
		// ./target/debug/subkey inspect "$SECRET//setheum//root"
		//
		// ORACLE
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//oracle"
		//
		// VALIDATOR 1
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//1//validator"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//1//babe"
		// ./target/debug/subkey --ed25519 inspect "$SECRET//setheum//1//grandpa"
		//
		// VALIDATOR 2
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//2//validator"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//2//babe"
		// ./target/debug/subkey --ed25519 inspect "$SECRET//setheum//2//grandpa"
		//
		move || {
			setheum_genesis(
				wasm_binary,
				// Initial PoA authorities (ROOT)
				vec![
					(
						// ROOT (sr25519)
						// 5CLg63YpPJNqcyWaYebk3LuuUVp3un7y1tmuV3prhdbnMA77
						hex!["0x683e1edef2dacd5996521a26617049cc5e7a8f10ebec2e664e2220329f986372"].into(),
						hex!["0x683e1edef2dacd5996521a26617049cc5e7a8f10ebec2e664e2220329f986372"].into(),
						hex!["21b5a771b99ef0f059c476502c018c4b817fb0e48858e95a238850d2b7828556"].unchecked_into(),
						hex!["948f15728a5fd66e36503c048cc7b448cb360a825240c48ff3f89efe050de608"].unchecked_into(),
					),
					(
						// ROOT (ed25519)
						// 5FnLzAUmXeTZg5J9Ao5psKU68oA5PBekXqhrZCKDbhSCQi88
						hex!["a476c0050065dafac1e9ff7bf602fe628ceadacf67650f8317554bd571b73507"].into(),
						hex!["a476c0050065dafac1e9ff7bf602fe628ceadacf67650f8317554bd571b73507"].into(),
						hex!["77f3c27e98da7849ed0749e1dea449321a4a5a36a1dccf3f08fc0ab3af24c62e"].unchecked_into(),
						hex!["b4f5713322656d29930aa89efa5509554a36c40fb50a226eae0f38fc1a6ceb25"].unchecked_into(),
					),
				],
				// Initial Oracle
				// 5F98oWfz2r5rcRVnP9VCndg33DAAsky3iuoBSpaPUbgN9AJn
				hex!["0x683e1edef2dacd5996521a26617049cc5e7a8f10ebec2e664e2220329f986372"].into(),

				// Initial Validators
				vec![
					// Validator 1
					// 5F98oWfz2r5rcRVnP9VCndg33DAAsky3iuoBSpaPUbgN9AJn
					hex!["0x683e1edef2dacd5996521a26617049cc5e7a8f10ebec2e664e2220329f986372"].into(),
					// Validator 2
					// 5Fe3jZRbKes6aeuQ6HkcTvQeNhkkRPTXBwmNkuAPoimGEv45
					hex!["9e22b64c980329ada2b46a783623bcf1f1d0418f6a2b5fbfb7fb68dbac5abf0f"].into(),
				],
			)
		},
		// Bootnodes
		vec![
			//TODO
			// "/dns/testnet-bootnode-1.ds3.setheum.xyz/blabla"
			// 	.parse()
			// 	.unwrap(
		],
		// Telemetry
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		// Protocol ID
		Some("setheum"),
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo".into(),
			para_id: 258_u32.into(),
		},
	))
}

/// Configure initial storage state for FRAME modules.
fn setheum_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> setheum_runtime::GenesisConfig {
	use setheum_runtime::{
		cent, dollar, get_all_module_accounts, SetheumOracleConfig, Balance, BalancesConfig, SecondOracleConfig,
		DexConfig, EnabledTradingPairs, GeneralCouncilMembershipConfig,
		MonetaryCouncilMembershipConfig, FinancialCouncilMembershipConfig, IndicesConfig, NativeTokenExistentialDeposit, 
		OperatorMembershipSetheumConfig, OperatorMembershipSecondConfig, OrmlNFTConfig, ParachainInfoConfig, 
		SudoConfig, SystemConfig, TechnicalCommitteeMembershipConfig, TokensConfig, VestingConfig, 
		DNAR, USDJ, EURJ, GBPJ, IDRJ, NGNJ, SETT, SDEX, DOT,
	};
	#[cfg(feature = "std")]
	use sp_std::collections::btree_map::BTreeMap;

	let existential_deposit = NativeTokenExistentialDeposit::get();

	let airdrop_accounts_json = &include_bytes!("../../../../../resources/newrome-airdrop-DNAR.json")[..];
	let airdrop_accounts: Vec<(AccountId, Balance)> = serde_json::from_slice(airdrop_accounts_json).unwrap();

	let initial_balance: u128 = 1_000_000 * dollar(DNAR);
	let initial_staking: u128 = 100_000 * dollar(DNAR);

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), initial_staking + dollar(DNAR))) // bit more for fee
		.chain(endowed_accounts.iter().cloned().map(|k| (k, initial_balance)))
		.chain(
			get_all_module_accounts()
				.iter()
				.map(|x| (x.clone(), existential_deposit)),
		)
		.chain(airdrop_accounts)
		.fold(
			BTreeMap::<AccountId, Balance>::new(),
			|mut acc, (account_id, amount)| {
				if let Some(balance) = acc.get_mut(&account_id) {
					*balance = balance
						.checked_add(amount)
						.expect("balance cannot overflow when building genesis");
				} else {
					acc.insert(account_id.clone(), amount);
				}
				acc
			},
		)
		.into_iter()
		.collect::<Vec<(AccountId, Balance)>>();

	setheum_runtime::GenesisConfig {
		frame_system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_indices: IndicesConfig { indices: vec![] },
		pallet_balances: BalancesConfig { balances },
		pallet_sudo: SudoConfig { key: root_key.clone() },
		pallet_collective_Instance1: Default::default(),
		pallet_membership_Instance1: GeneralCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},

		//TODO: Add the Shura Council
		//TODO: Add and Rename SerpCouncil to MonetaryCouncil
		pallet_collective_Instance2: Default::default(),
		pallet_membership_Instance2: MonetaryCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_collective_Instance3: Default::default(),
		pallet_membership_Instance3: FinancialCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_collective_Instance4: Default::default(),
		pallet_membership_Instance4: TechnicalCommitteeMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_membership_Instance5: OperatorMembershipSetheumConfig {
			members: endowed_accounts.clone(),
			phantom: Default::default(),
		},
		pallet_membership_Instance6: OperatorMembershipSecondConfig {
			members: endowed_accounts.clone(),
			phantom: Default::default(),
		},
		pallet_treasury: Default::default(),
		orml_tokens: TokensConfig {
			endowed_accounts: vec![
				(root_key.clone(), DOT, initial_balance),
				(root_key, JGBP, initial_balance),
			],
		},
		orml_oracle_Instance1: SetheumOracleConfig {
			members: Default::default(), // initialized by OperatorMembership
			phantom: Default::default(),
		},
		orml_oracle_Instance2: SecondOracleConfig {
			members: Default::default(), // initialized by OperatorMembership
			phantom: Default::default(),
		},
		setheum_dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		},
		parachain_info: ParachainInfoConfig {
			parachain_id: 666.into(),
		},
		orml_nft: OrmlNFTConfig { tokens: vec![] },
	}
}
