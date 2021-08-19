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

use 
use setheum_primitives::AccountId;
use hex_literal::hex;
use sc_chain_spec::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde_json::map::Map;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{traits::Zero, FixedPointNumber, FixedU128, Perbill};
use sp_std::collections::btree_map::BTreeMap;

use crate::chain_spec::{Extensions, TELEMETRY_URL};
use setheum_runtime::{
	cent, dollar, get_all_module_accounts, SetheumOracleConfig, BabeConfig, Balance, BalancesConfig,
	DexConfig, EnabledTradingPairs,
	GeneralCouncilMembershipConfig, SetheumJuryMembershipConfig, GrandpaConfig,
	FinancialCouncilMembershipConfig, ExchangeCouncilMembershipConfig, IndicesConfig,
	NativeTokenExistentialDeposit, OperatorMembershipSetheumConfig, OrmlNFTConfig,
	RenVmBridgeConfig, SessionConfig, StakerStatus, StakingConfig, SudoConfig,
	SystemConfig, TechnicalCommitteeMembershipConfig, TokensConfig, VestingConfig,
	DNAR, DRAM, SETR, USDJ, EURJ, JPYJ, GBPJ, AUDJ, CADJ, CHFJ, SEKJ, SGDJ, SARJ RENBTC,
};
use runtime_common::TokenInfo;

pub type ChainSpec = sc_service::GenericChainSpec<setheum_runtime::GenesisConfig, Extensions>;

fn setheum_session_keys(
	grandpa: GrandpaId,
	babe: BabeId) -> setheum_runtime::SessionKeys {
	setheum_runtime::SessionKeys { grandpa, babe }
}

// pub fn setheum_config() -> Result<ChainSpec, String> {
// 	Err("Not available".into())
// }

pub fn setheum_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../../../resources/setheum-dist.json")[..])
}

fn setheum_properties() -> Properties {
	let mut properties = Map::new();
	let mut token_symbol: Vec<String> = vec![];
	let mut token_decimals: Vec<u32> = vec![];
	[
		DNAR, DRAM, SETR, USDJ, EURJ, JPYJ, GBPJ, AUDJ, CADJ, CHFJ, SEKJ, SGDJ, SARJ RENBTC,
	].iter().for_each(|token| {
		token_symbol.push(token.symbol().unwrap().to_string());
		token_decimals.push(token.decimals().unwrap() as u32);
	});
	properties.insert("tokenSymbol".into(), token_symbol.into());
	properties.insert("tokenDecimals".into(), token_decimals.into());

	properties
}

pub fn latest_setheum_config() -> Result<ChainSpec, String> {
	let wasm_binary = setheum_runtime::WASM_BINARY.ok_or("Setheum runtime wasm binary not available")?;

	Ok(ChainSpec::from_genesis(
		"Setheum Goldnet",
		"Setheum",
		ChainType::Live,
		// SECRET="..."
		// ./target/debug/subkey inspect "$SECRET//setheum//root"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//oracle"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//1//validator"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//1//babe"
		// ./target/debug/subkey --ed25519 inspect "$SECRET//setheum//1//grandpa"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//2//validator"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//2//babe"
		// ./target/debug/subkey --ed25519 inspect "$SECRET//setheum//2//grandpa"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//3//validator"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//3//babe"
		// ./target/debug/subkey --ed25519 inspect "$SECRET//setheum//3//grandpa"
		move || {
			let existential_deposit = NativeTokenExistentialDeposit::get();
			let mut total_allocated: Balance = Zero::zero();

			let airdrop_accounts_json = &include_bytes!("../../../../resources/newrome-airdrop-DNAR.json")[..];
			let airdrop_accounts: Vec<(AccountId, Balance)> = serde_json::from_slice(airdrop_accounts_json).unwrap();
			let other_allocation_json = &include_bytes!("../../../../resources/setheum-allocation-DNAR.json")[..];
			let other_allocation: Vec<(AccountId, Balance)> = serde_json::from_slice(other_allocation_json).unwrap();
			// TODO: Update to add `setheum-allocation-SETR.json` and `setheum-allocation-DRAM.json` too.

			// TODO: Update!
			// Initial PoA authorities
			let initial_authorities: Vec<(AccountId, AuraId)> = vec![
				(
					// Authority 1
					// 5CLg63YpPJNqcyWaYebk3LuuUVp3un7y1tmuV3prhdbnMA77
					hex!["0c2df85f943312fc853059336627d0b7a08669629ebd99b4debc6e58c1b35c2b"].into(),
					hex!["0c2df85f943312fc853059336627d0b7a08669629ebd99b4debc6e58c1b35c2b"].into(),
					hex!["21b5a771b99ef0f059c476502c018c4b817fb0e48858e95a238850d2b7828556"].unchecked_into(),
					hex!["948f15728a5fd66e36503c048cc7b448cb360a825240c48ff3f89efe050de608"].unchecked_into(),
				),
				(
					// Authority 3
					// 5Gn5LuLuWNcY21Vue4QcFFD3hLvjQY3weMHXuEyejUbUnArt
					hex!["d07e538fee7c42be9b2627ea5caac9a30f1869d65af2a19df70138d5fcc34310"].into(),
					hex!["d07e538fee7c42be9b2627ea5caac9a30f1869d65af2a19df70138d5fcc34310"].into(),
					hex!["c5dfcf68ccf1a64ed4145383e4bbbb8bbcc50f654d87187c39df2b88a9683b7f"].unchecked_into(),
					hex!["4cc54799f38715771605a21e8272a7a1344667e4681611988a913412755a8a04"].unchecked_into(),
				),
				(
					// Authority 3
					// 5Gn5LuLuWNcY21Vue4QcFFD3hLvjQY3weMHXuEyejUbUnArt
					hex!["d07e538fee7c42be9b2627ea5caac9a30f1869d65af2a19df70138d5fcc34310"].into(),
					hex!["d07e538fee7c42be9b2627ea5caac9a30f1869d65af2a19df70138d5fcc34310"].into(),
					hex!["c5dfcf68ccf1a64ed4145383e4bbbb8bbcc50f654d87187c39df2b88a9683b7f"].unchecked_into(),
					hex!["4cc54799f38715771605a21e8272a7a1344667e4681611988a913412755a8a04"].unchecked_into(),
				),
			];

			// TODO: Update!
			// General Council members
			let general_councils: Vec<AccountId> = vec![
				// Member 1
				// ouJX1WJQ9s4RMukAx5zvMwPY2zJZ9Xr5euzRG97Ne6UTNG9
				hex!["1ab677fa2007fb1e8ac2f5f6d253d5a2bd9c2ed4e5d3c1565c5d84436f81325d"].into(),
				// Member 2
				// qMJYLJEP2HTBFhxqTFAJz9RcsT9UQ3VW2tFHRBmyaxPdj1n
				hex!["5ac728d31a0046274f1c5bece1867555c6728c8e8219ff77bb7a8afef4ab8137"].into(),
				// Member 3
				// qPnkT89PRdiCbBgvE6a6gLcFCqWC8F1UoCZUhFvjbBkXMXc
				hex!["5cac9c2837017a40f90cc15b292acdf1ee28ae03005dff8d13d32fdf7d2e237c"].into(),
				// Member 4
				// sZCH1stvMnSuDK1EDpdNepMYcpZWoDt3yF3PnUENS21f2tA
				hex!["1ab677fa2007fb1e8ac2f5f6d253d5a2bd9c2ed4e5d3c1565c5d84436f81325d"].into(),
				// Member 5
				// ra6MmAYU2qdCVsMS3REKZ82CJ1EwMWq6H6Zo475xTzedctJ
				hex!["90c492f38270b5512370886c392ff6ec7624b14185b4b610b30248a28c94c953"].into(),
				// Member 6
				// ts9q95ZJmaCMCPKuKTY4g5ZeK65GdFVz6ZDD8LEnYJ3jpbm
				hex!["f63fe694d0c8a0703fc45362efc2852c8b8c9c4061b5f0cf9bd0329a984fc95d"].into(),
			];

			// TODO: Update!
			// Setheum Jury members
			let setheum_jury: Vec<AccountId> = vec![
				// Member 1
				// ouJX1WJQ9s4RMukAx5zvMwPY2zJZ9Xr5euzRG97Ne6UTNG9
				hex!["1ab677fa2007fb1e8ac2f5f6d253d5a2bd9c2ed4e5d3c1565c5d84436f81325d"].into(),
				// Member 2
				// qMJYLJEP2HTBFhxqTFAJz9RcsT9UQ3VW2tFHRBmyaxPdj1n
				hex!["5ac728d31a0046274f1c5bece1867555c6728c8e8219ff77bb7a8afef4ab8137"].into(),
				// Member 3
				// qPnkT89PRdiCbBgvE6a6gLcFCqWC8F1UoCZUhFvjbBkXMXc
				hex!["5cac9c2837017a40f90cc15b292acdf1ee28ae03005dff8d13d32fdf7d2e237c"].into(),
				// Member 4
				// sZCH1stvMnSuDK1EDpdNepMYcpZWoDt3yF3PnUENS21f2tA
				hex!["1ab677fa2007fb1e8ac2f5f6d253d5a2bd9c2ed4e5d3c1565c5d84436f81325d"].into(),
				// Member 5
				// ra6MmAYU2qdCVsMS3REKZ82CJ1EwMWq6H6Zo475xTzedctJ
				hex!["90c492f38270b5512370886c392ff6ec7624b14185b4b610b30248a28c94c953"].into(),
				// Member 6
				// ts9q95ZJmaCMCPKuKTY4g5ZeK65GdFVz6ZDD8LEnYJ3jpbm
				hex!["f63fe694d0c8a0703fc45362efc2852c8b8c9c4061b5f0cf9bd0329a984fc95d"].into(),
			];

			// TODO: Update!
			// sWcq8FAQXPdXGSaxSTBKS614hCB8YutkVWWacBKG1GbGS23
			let root_key: AccountId = hex!["ba5a672d05b5db2ff433ee3dc24cf021e301bc9d44232046ce7bd45a9360fa50"].into();

			let initial_allocation = initial_authorities
				.iter()
				.map(|x| (x.0.clone(), existential_deposit))
				.chain(airdrop_accounts)
				.chain(other_allocation)
				.chain(
					get_all_module_accounts()
						.iter()
						.map(|x| (x.clone(), existential_deposit)), // add ED for module accounts
				)
				.fold(
					BTreeMap::<AccountId, Balance>::new(),
					|mut acc, (account_id, amount)| {
						// merge duplicated accounts
						if let Some(balance) = acc.get_mut(&account_id) {
							*balance = balance
								.checked_add(amount)
								.expect("balance cannot overflow when building genesis");
						} else {
							acc.insert(account_id.clone(), amount);
						}

						total_allocated = total_allocated
							.checked_add(amount)
							.expect("total insurance cannot overflow when building genesis");
						acc
					},
				)
				.into_iter()
				.collect::<Vec<(AccountId, Balance)>>();

			// TODO: Update!
			// check total allocated
			assert_eq!(
				total_allocated,
				258_000_000 * dollar(DNAR), // 258 million DNAR
				"total allocation must be equal to 258 million DNAR"
			);

			// TODO: Update to add `setheum-vesting-SETR.json` and `setheum-vesting-DRAM.json` too.
			let vesting_list_json = &include_bytes!("../../../../resources/setheum-vesting-DNAR.json")[..];
			let vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)> =
				serde_json::from_slice(vesting_list_json).unwrap();

			// ensure no duplicates exist.
			let unique_vesting_accounts = vesting_list
				.iter()
				.map(|(x, _, _, _, _)| x)
				.cloned()
				.collect::<std::collections::BTreeSet<_>>();
			assert!(
				unique_vesting_accounts.len() == vesting_list.len(),
				"duplicate vesting accounts in genesis."
			);

			setheum_genesis(
				wasm_binary,
				initial_authorities,
				root_key,
				initial_allocation,
				vesting_list,
				general_councils,
				setheum_jury,
			)
			// TODO: Update!
			setheum_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					(
						// Authority 1
						// 5CLg63YpPJNqcyWaYebk3LuuUVp3un7y1tmuV3prhdbnMA77
						hex!["0c2df85f943312fc853059336627d0b7a08669629ebd99b4debc6e58c1b35c2b"].into(),
						hex!["0c2df85f943312fc853059336627d0b7a08669629ebd99b4debc6e58c1b35c2b"].into(),
						hex!["21b5a771b99ef0f059c476502c018c4b817fb0e48858e95a238850d2b7828556"].unchecked_into(),
						hex!["948f15728a5fd66e36503c048cc7b448cb360a825240c48ff3f89efe050de608"].unchecked_into(),
					),
					(
						// Authority 2
						// 5FnLzAUmXeTZg5J9Ao5psKU68oA5PBekXqhrZCKDbhSCQi88
						hex!["a476c0050065dafac1e9ff7bf602fe628ceadacf67650f8317554bd571b73507"].into(),
						hex!["a476c0050065dafac1e9ff7bf602fe628ceadacf67650f8317554bd571b73507"].into(),
						hex!["77f3c27e98da7849ed0749e1dea449321a4a5a36a1dccf3f08fc0ab3af24c62e"].unchecked_into(),
						hex!["b4f5713322656d29930aa89efa5509554a36c40fb50a226eae0f38fc1a6ceb25"].unchecked_into(),
					),
					(
						// Authority 3
						// 5Gn5LuLuWNcY21Vue4QcFFD3hLvjQY3weMHXuEyejUbUnArt
						hex!["d07e538fee7c42be9b2627ea5caac9a30f1869d65af2a19df70138d5fcc34310"].into(),
						hex!["d07e538fee7c42be9b2627ea5caac9a30f1869d65af2a19df70138d5fcc34310"].into(),
						hex!["c5dfcf68ccf1a64ed4145383e4bbbb8bbcc50f654d87187c39df2b88a9683b7f"].unchecked_into(),
						hex!["4cc54799f38715771605a21e8272a7a1344667e4681611988a913412755a8a04"].unchecked_into(),
					),
				],
				// Sudo account
				// 5F98oWfz2r5rcRVnP9VCndg33DAAsky3iuoBSpaPUbgN9AJn
				hex!["8815a8024b06a5b4c8703418f52125c923f939a5c40a717f6ae3011ba7719019"].into(),
				// Pre-funded accounts
				vec![
					// Pre-funded account 1 (same as sudo account here)
					// 5F98oWfz2r5rcRVnP9VCndg33DAAsky3iuoBSpaPUbgN9AJn
					hex!["8815a8024b06a5b4c8703418f52125c923f939a5c40a717f6ae3011ba7719019"].into(),
					// Pre-funded account 2
					// 5Fe3jZRbKes6aeuQ6HkcTvQeNhkkRPTXBwmNkuAPoimGEv45
					hex!["9e22b64c980329ada2b46a783623bcf1f1d0418f6a2b5fbfb7fb68dbac5abf0f"].into(),
				],
			)
		},
		vec![
			//TODO Update!
			//"/dns/testnet-bootnode-1.setheum.setheum.xyz/tcp/30333/p2p/12D3KooWAFUNUowRqCV4c5so58Q8iGpypVf3L5ak91WrHf7rPuKz"
			//	.parse()
			//	.unwrap(),
		],
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		Some("setheum"),
		Some(properties),
		Default::default(),
	))
}

//TODO Update!
fn setheum_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
	root_key: AccountId,
	initial_allocation: Vec<(AccountId, Balance)>,
	vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)>,
	general_councils: Vec<AccountId>,
	setheum_jury: Vec<AccountId>,
) -> setheum_runtime::GenesisConfig {
	setheum_runtime::GenesisConfig {
		frame_system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: BalancesConfig {
			balances: initial_allocation,
		},
		pallet_indices: Some(IndicesConfig { indices: vec![] }),
		pallet_session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.0.clone(), setheum_session_keys(x.2.clone(), x.3.clone())))
				.collect::<Vec<_>>(),
		},
		serp_staking: StakingConfig {
			validator_count: 5,
			minimum_validator_count: 1,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), initial_staking, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		pallet_sudo: SudoConfig { key: root_key },
		pallet_babe: BabeConfig { authorities: initial_authorities },
		pallet_grandpa: GrandpaConfig { authorities: initial_authorities },
		pallet_collective_Instance1: Default::default(),
		pallet_membership_Instance1: GeneralCouncilMembershipConfig {
			members: general_councils,
			phantom: Default::default(),
		},
		pallet_collective_Instance2: Default::default(),
		pallet_membership_Instance2: SetheumJuryMembershipConfig {
			members: setheum_jury,
			phantom: Default::default(),
		},
		pallet_collective_Instance3: Default::default(),
		pallet_membership_Instance3: FinancialCouncilMembershipConfig {
			members: vec![],
			phantom: Default::default(),
		},
		pallet_collective_Instance4: Default::default(),
		pallet_membership_Instance4: ExchangeCouncilMembershipConfig {
			members: vec![],
			phantom: Default::default(),
		},
		pallet_collective_Instance5: Default::default(),
		pallet_membership_Instance5: TechnicalCommitteeMembershipConfig {
			members: vec![],
			phantom: Default::default(),
		},
		pallet_membership_Instance6: OperatorMembershipSetheumConfig {
			members: endowed_accounts.clone(),
			phantom: Default::default(),
		},
		pallet_treasury: Default::default(),
		orml_tokens: TokensConfig {
			endowed_accounts: vec![],
		},
		orml_vesting: VestingConfig { vesting: vesting_list },
		orml_oracle_Instance1: SetheumOracleConfig {
			members: Default::default(), // initialized by OperatorMembership
			phantom: Default::default(),
		},
		setheum_evm: Default::default(),
		setheum_renvm_bridge: RenVmBridgeConfig {
			ren_vm_public_key: hex!["4b939fc8ade87cb50b78987b1dda927460dc456a"],
		},
		setheum_dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		},
		orml_nft: OrmlNFTConfig { tokens: vec![] },
	}
}
