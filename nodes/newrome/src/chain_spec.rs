// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

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

use sp_core::{Pair, Public, sr25519, H160, Bytes};
use newrome_runtime::{
	//
	AccountId,
	//
	BabeConfig, BalancesConfig, GenesisConfig, SystemConfig,
	SS58Prefix, opaque::SessionKeys, get_all_module_accounts,
	ImOnlineId, IndicesConfig, SessionConfig, StakingConfig,
	AuthorityDiscoveryId, EVMConfig, AuthorityDiscoveryConfig,
	StakerStatus,  VestingConfig,
	//
	SudoConfig,
	ShuraCouncilMembershipConfig,
	FinancialCouncilMembershipConfig,
	TechnicalCommitteeMembershipConfig,
	OperatorMembershipSetheumConfig,
	CdpTreasuryConfig,
	CdpEngineConfig,

	//
	DexConfig, EnabledTradingPairs,
	TokensConfig, OrmlNFTConfig,
	NativeTokenExistentialDeposit, MaxNativeTokenExistentialDeposit,
	//
	SETM, ETH, WBTC, BNB, USDW, USDI,
};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount};
use sp_runtime::{traits::Zero, FixedPointNumber, FixedU128};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;

use sp_std::{collections::btree_map::BTreeMap, str::FromStr};
use sc_chain_spec::ChainSpecExtension;
use serde_json::map::Map;

use serde::{Deserialize, Serialize};

use hex_literal::hex;
use sp_core::{crypto::UncheckedInto, bytes::from_hex};

use system_primitives::{AccountPublic, Balance, Nonce, currency::TokenInfo, TradingPair};
use newrome_runtime::BABE_GENESIS_EPOCH_CONFIG;

// The URL for the telemetry server.
const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<system_primitives::Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<system_primitives::Block>,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

fn get_session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
	) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online, authority_discovery }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an authority keys.
pub fn get_authority_keys_from_seed(seed: &str)
	-> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = newrome_runtime::WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || dev_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				get_authority_keys_from_seed("Alice"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			]
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(newrome_properties()),
		// Extensions
		Default::default(),
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = newrome_runtime::WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || dev_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				get_authority_keys_from_seed("Alice"),
				get_authority_keys_from_seed("Bob"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
			]
		),
		// Bootnodes
		vec![],
		// Telemetry
		// TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		None,
		// Protocol ID
		Some("newrome_local_testnet"),
		// Properties
		Some(newrome_properties()),
		// Extensions
		Default::default(),
	))
}

pub fn public_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = newrome_runtime::WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Newrome Testnet",
		// ID
		"newrome_testnet",
		ChainType::Live,
		move || testnet_genesis(
			wasm_binary,
			// Initial authorities keys:
			// stash
			// controller
			// grandpa
			// babe
			// im-online
			// authority-discovery
			//
			// for i in 1; do for j in stash; do subkey inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in controller; do subkey inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in grandpa; do subkey --ed25519 inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in babe; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in im_online; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in authority_discovery; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
			//
			vec![
				(	// v1 stash: VQgV4scizieRZ6s3qbDCM1RMcG4TfKuGPiwWNGR2Bk71gvUrq
					hex!["2699d78960802127b10ebbd092039bbbe11383c66ab10c7413bb5f712f695c3c"].into(),
					// v1 controller: VQjfJyGz5FtxenPuG9XYtdfKZhJAtUAwRz34CGGDbSfBotsis
					hex!["b31e73f9a4366fe1c2a34c1ec7362c217f9047ff77455772c640098da4323eaa"].into(),
					// v1 grandpa: VQk9KbBUkXivNDC7cbExSW5mqYuxR44yzcmCWsqjfAaKevDES
					hex!["c87b6884473bab40ae10181d19e9db0de1443380a68e8905bc1430e101b24f80"].unchecked_into(),
					// v1 babe: VQhRPhrSZbtYRadjxyxJuXMKEQwjm6VfvB5YBg43KuBjwGVkE
					hex!["50087c9f582c4043891c1becc399e21cddea7be7e60e378d4c15afd2d47d6f23"].unchecked_into(),
					// v1 im_online: VQkQSMT5WxcsJuHjLMnpezv7ir3VaQQDi6iCBbXRkihC9qimn
					hex!["d402db42e11b929b543c3aa28fa0da9d340d81e0a64e38b0dbb23cdf96327746"].unchecked_into(),
					// v1 authority_discovery: VQk3yaX8GMezXveu8cAu4o7DzSnQsan22ZLmqdoWokDjcedhJ
					hex!["c467d11e1bbed656e85d3ca6ba48ccdb7473963c28594318306d10033955300d"].unchecked_into(),
				),
				(	// v2 stash: VQhXeeVvmWA9XMfQSPYyEjitn71Pmrqm5B3zTVLXNaDwTy5VZ
					hex!["54ce4253869a4b3e0490ef8c815049ca3fa519586e2d94012a5e95d283e9de5e"].into(),
					// v2 controller: VQhy8nbbdsrCLQLmJA9kbGQacXXRAxM9GbM2zEWRREJHYwm3C
					hex!["683e1edef2dacd5996521a26617049cc5e7a8f10ebec2e664e2220329f986372"].into(),
					// v2 grandpa: VQiausGmcMyWV83dX4FQuyHDo58d5KF7n6pAH7qESCbcGX1dr
					hex!["838776f0364051518fbb74e15a4d2ba174b9ec4e67365e496858581198dcf900"].unchecked_into(),
					// v2 babe: VQkWsg1j2Ps1Vank78vkr9fuMdJjKNZPLENZx79yi1otwaFdr
					hex!["d8eb90553458168fd40a0bbb8baa683f61090558e4c02d9165018e7dc5826d7f"].unchecked_into(),
					// v2 im_online: VQg8YSx7VbWUdNsXK3Y35VmeywyUwR93RBjQgQqrNP9jSQUdL
					hex!["16f2836505508630e3da0e2a792836570011026a4259c95b8c2f5be913935a07"].unchecked_into(),
					// v2 authority_discovery: VQgr5YmfAR4Bj2hkNU7z4KQDFPe9Gswsyp2G3RgTuYAyJY9vj
					hex!["36a04496492bbfbf6d559618c0f30d69cbc5f7c6f3b87e6271e0ea1af027f209"].unchecked_into(),
				),
				(	// v3 stash: VQkF7GzpAdvdzeXr6ktF9QdyHgG4pZFLQtnjjByawkxWygNze
					hex!["cce56e014c7be762154826d13888546d3e16a22197f932950107bda8b58cd06e"].into(),
					// v3 controller: VQkUHcac6h8kYXU9Yc78v1d9rP9QqFMAPPq7b6ejGHzLJQYnn
					hex!["d6f26b6688766de94dec42a25cc3a3b97c275732d5e03de8fd7afa2c3897990e"].into(),
					// v3 grandpa: VQm9tmM6FqyTtdAriXr1YkLVRHLBWNoZXx98akhqZApT9z7s5
					hex!["f52751d3df56977b1d8e4af180ed5cf68ff37a4a98a6ecf038210f4162d075b4"].unchecked_into(),
					// v3 babe: VQhvQzoGURhcrtSzJCjBYwWnF3Tw6YicJRe7bd8YhFXqr3cFd
					hex!["662af40d803294fc32868b730878b369ad0436f8b15ab6cf5d47762059c5923b"].unchecked_into(),
					// v3 im_online: VQm45RcteXKwyJPKia2En5ghen5n6Z6B6DF4P6K116NkVgLCR
					hex!["f0b7bb45a3555d4681b2a59e8852b746ecfa8f601a101b2d5d160f454eb3803e"].unchecked_into(),
					// v3 authority_discovery: VQhHraEqKuBbfypk9Cah7k3JitcgJawEZZ6htzJUgHa5RyFCA
					hex!["4a48f8c587d263c6c2c6cb510bd8d565af2a1e87d7e35d12dc8e0a5ff078a759"].unchecked_into(),
				),
			],
			// Sudo: VQgyc63yJgmrhrsDfH73ipq6TfEyiPMNQ3QYK3a82Sskb3mFx
			hex!["3c5dca516188b2ac077e33a886ac1ea2c03d2a157f56b70ca182c9f7fe5f9055"].into(),
			// --------------------------------------------------------------------------------------------
			// Endowed accounts vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
			//
			vec![
				// Foundation & Airdrop: VQh5AghroqszPazCMEdpN8QkAMvXCTkXbANx6GSqFz2kTebuv
				(hex!["409bc00c7f4d8cf046c1eb363022eec1103e70ae180cba92056452315837c71a"].into(), 783_250_000 as u128),
				// Treasury: VQi5wfR3UZnJY4KvxG2uVPP6n6CS296xTznrzqs3hfD4Fyp6x
				(hex!["6d6f646c7365742f747273790000000000000000000000000000000000000000"].into(), 313_300_000 as u128),
				// Team and DEX Liquidity Offering Fund: VQgPxsHbvGdXC7HhUvYvPifu1SyAuRnUhbMw4hAaTm9fwvkkz
				(hex!["22b565e2303579c0d50884a3524c32ed12c8b91a8621dd72270b8fd17d20d009"].into(), 1_566_500_000 as u128),
			],
			// Foundation & Airdrop: VQh5AghroqszPazCMEdpN8QkAMvXCTkXbANx6GSqFz2kTebuv
			hex!["409bc00c7f4d8cf046c1eb363022eec1103e70ae180cba92056452315837c71a"].into(),
			// Team and DEX Liquidity Offering Fund: VQgPxsHbvGdXC7HhUvYvPifu1SyAuRnUhbMw4hAaTm9fwvkkz
			hex!["22b565e2303579c0d50884a3524c32ed12c8b91a8621dd72270b8fd17d20d009"].into(),
		),
		// Bootnodes - TODO: Update!
		vec![],
		// Telemetry
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		// Protocol ID
		Some("newrome_testnet"),
		// Properties
		Some(newrome_properties()),
		// Extensions
		Default::default(),
	))
}


pub fn live_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../../resources/chain_spec_newrome_raw.json")[..])
}

fn dev_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {

	let existential_deposit = NativeTokenExistentialDeposit::get();

	let evm_genesis_accounts = evm_genesis();

	let initial_balance: u128 = 10_000 * 1_000_000_000_000_000_000;	// 1,000,000 SETM/ETH/WBTC/BNB/USDW/USDI
	let initial_staking: u128 = 2_000 * 1_000_000_000_000_000_000; 	// 258,000 SETM/ETH/WBTC/BNB/USDW/USDI

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), initial_staking + 1_000_000_000_000_000_000))
		.chain(endowed_accounts.iter().cloned().map(|k| (k, initial_balance)))
		.chain(
			get_all_module_accounts()
				.iter()
				.map(|x| (x.clone(), existential_deposit)),
		)
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

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		indices: IndicesConfig { indices: vec![] },
		balances: BalancesConfig { balances },
		sudo: SudoConfig { key: root_key.clone() },
		shura_council: Default::default(),
		shura_council_membership: ShuraCouncilMembershipConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
			],
			phantom: Default::default(),
		},
		financial_council: Default::default(),
		financial_council_membership: FinancialCouncilMembershipConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
			],
			phantom: Default::default(),
		},
		technical_committee: Default::default(),
		technical_committee_membership: TechnicalCommitteeMembershipConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
			],
			phantom: Default::default(),
		},
		operator_membership_setheum: OperatorMembershipSetheumConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
			],
			phantom: Default::default(),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (
						x.0.clone(), // stash
						x.0.clone(), // stash
						get_session_keys(
							x.2.clone(), // grandpa
							x.3.clone(), // babe
							x.4.clone(), // im-online
							x.5.clone(), // authority-discovery
						)))
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32 + 2,
			minimum_validator_count: 1 as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), initial_staking, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		},
		babe: BabeConfig { authorities: Default::default(), epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG) },
		grandpa: Default::default(),
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		im_online: Default::default(),
		treasury: Default::default(), // Main Treasury (Newrome Treasury)
		tokens: TokensConfig {
			balances: endowed_accounts
				.iter()
				.flat_map(|x| vec![
					(x.clone(), ETH, initial_balance),
					(x.clone(), WBTC, initial_balance),
					(x.clone(), BNB, initial_balance),
					(x.clone(), USDW, initial_balance),
					(x.clone(), USDI, initial_balance),
					])
				.collect(),
		},
		evm: EVMConfig {
			accounts: evm_genesis_accounts,
			treasury: root_key,
		},
		vesting: VestingConfig {
			vesting: endowed_accounts
			.iter()
			.flat_map(|x| vec![
				(x.clone(), SETM, 10, 1, 3600, 100_000_000_000_000_000),
				(x.clone(), ETH, 10, 1, 3600, 100_000_000_000_000_000),
				(x.clone(), WBTC, 10, 1, 3600, 100_000_000_000_000_000),
				(x.clone(), BNB, 10, 1, 3600, 100_000_000_000_000_000),
				(x.clone(), USDW, 10, 1, 3600, 100_000_000_000_000_000),
				(x.clone(), USDI, 10, 1, 3600, 100_000_000_000_000_000),
				])
			.collect(),
		},
		cdp_treasury: CdpTreasuryConfig {
			expected_collateral_auction_size: vec![
				(SETM, 500 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(ETH, 300 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(WBTC, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(BNB, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
			],
		},
		cdp_engine: CdpEngineConfig {
			collaterals_params: vec![
				(
					SETM,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in USDI (cap)
				),
				(
					ETH,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in USDI (cap)
				),
				(
					WBTC,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in USDI (cap)
				),
				(
					BNB,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in USDI (cap)
				),
			],
		},
		dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		},
		orml_nft: OrmlNFTConfig { tokens: vec![] }
	}
}

fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, Balance)>,
	foundation: AccountId,
	team: AccountId,
) -> GenesisConfig {

	// Allocations Endowment
	let  serp_foundation_alloc: u128 = 51_600_000 * 1_000_000_000_000_000_000;
	let  serp_team_alloc: u128 = 154_800_000 * 1_000_000_000_000_000_000;
	let  serp_airdrops_alloc: u128 = 12_900_000 * 1_000_000_000_000_000_000;

	let  dnar_foundation_alloc: u128 = 14_000_000 * 1_000_000_000_000_000_000;
	let  dnar_team_alloc: u128 = 42_000_000 * 1_000_000_000_000_000_000;
	let  dnar_airdrop_alloc: u128 = 3_500_000 * 1_000_000_000_000_000_000;
	
	let  help_foundation_alloc: u128 = 140_000_000 * 1_000_000_000_000_000_000;
	let  help_team_alloc: u128 = 420_000_000 * 1_000_000_000_000_000_000;
	let  help_airdrop_alloc: u128 = 35_000_000 * 1_000_000_000_000_000_000;

	let  setr_foundation_alloc: u128 = 200_000_000 * 1_000_000_000_000_000_000;
	let  setr_team_alloc: u128 = 1_000_000_000 * 1_000_000_000_000_000_000;
	let  setr_cashdrop_alloc: u128 = 200_000_000 * 1_000_000_000_000_000_000;
	let  setr_airdrop_alloc: u128 = 200_000_000 * 1_000_000_000_000_000_000;
	
	let  setusd_foundation_alloc: u128 = 31_330_000 * 1_000_000_000_000_000_000;
	let  setusd_team_alloc: u128 = 156_650_000 * 1_000_000_000_000_000_000;
	let  setusd_cashdrop_alloc: u128 = 31_330_000 * 1_000_000_000_000_000_000;
	let  setusd_airdrop_alloc: u128 = 31_330_000 * 1_000_000_000_000_000_000;

	// Allocations Vesting - per_period
	let  setm_foundation_vesting: u128 = 2_664_659_454_310_403_000;
	let  setm_team_vesting: u128 = 39_969_891_814_656_050_00;

	let  serp_foundation_vesting: u128 = 27_625_401_531_999_000;
	let  serp_team_vesting: u128 = 82_876_204_595_997_000;

	let  dnar_foundation_vesting: u128 = 74_952_639_815_501_000;
	let  dnar_team_vesting: u128 = 112_428_959_723_252_000;

	let  help_foundation_vesting: u128 = 749_526_398_155_010_000;
	let  help_team_vesting: u128 = 1_124_289_597_232_520_000;

	let  setr_foundation_vesting: u128 = 3_912_363_067_292_644_000;
	let  setr_team_vesting: u128 = 11_737_089_201_877_930_000;
	
	let  setusd_foundation_vesting: u128 = 766_089_593_114_241_000;
	let  setusd_team_vesting: u128 = 2_298_268_779_342_723_000;

	let existential_deposit = NativeTokenExistentialDeposit::get();

	let evm_genesis_accounts = evm_genesis();

	let  initial_staking: u128 = 100_000 * 1_000_000_000_000_000_000;

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), initial_staking + 1_000_000_000_000_000_000)) // bit more for fees
		.chain(endowed_accounts.iter().cloned().map(|x| (x.0.clone(), x.1 * 1_000_000_000_000_000_000)))
		.chain(
			get_all_module_accounts()
				.iter()
				.map(|x| (x.clone(), existential_deposit)),
		)
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

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		indices: IndicesConfig { indices: vec![] },
		balances: BalancesConfig { balances },
		sudo: SudoConfig { key: root_key.clone() },
		shura_council: Default::default(),
		shura_council_membership: ShuraCouncilMembershipConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
			],
			phantom: Default::default(),
		},
		financial_council: Default::default(),
		financial_council_membership: FinancialCouncilMembershipConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
			],
			phantom: Default::default(),
		},
		technical_committee: Default::default(),
		technical_committee_membership: TechnicalCommitteeMembershipConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
			],
			phantom: Default::default(),
		},
		operator_membership_setheum: OperatorMembershipSetheumConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
			],
			phantom: Default::default(),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (
						x.0.clone(), // stash
						x.0.clone(), // stash
						get_session_keys(
							x.2.clone(), // grandpa
							x.3.clone(), // babe
							x.4.clone(), // im-online
							x.5.clone(), // authority-discovery
						)))
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32 + 2,
			minimum_validator_count: 1 as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), initial_staking, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		},
		babe: BabeConfig { authorities: Default::default(), epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG) },
		grandpa: Default::default(),
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		im_online: Default::default(),
		treasury: Default::default(), // Main Treasury (Newrome Treasury)
		tokens: TokensConfig {
			balances: vec![
				(foundation.clone(), ETH, serp_foundation_alloc + serp_airdrops_alloc),
				(team.clone(), ETH, serp_team_alloc),
				(foundation.clone(), WBTC, dnar_foundation_alloc + dnar_airdrop_alloc),
				(team.clone(), WBTC, dnar_team_alloc),
				(foundation.clone(), BNB, help_foundation_alloc + help_airdrop_alloc),
				(team.clone(), BNB, help_team_alloc),
				(foundation.clone(), USDW, setr_foundation_alloc + setr_airdrop_alloc),
				(team.clone(), USDW, setr_team_alloc),
				(foundation.clone(), USDI, setusd_foundation_alloc + setusd_airdrop_alloc),
				(team.clone(), USDI, setusd_team_alloc),
			]
		},
		evm: EVMConfig {
			accounts: evm_genesis_accounts,
			treasury: root_key,
		},
		vesting: VestingConfig {
			vesting: vec![
				// All schedules here last 1 lunar year.
				(foundation.clone(), SETM, 258, 1, 5_112_000, setm_foundation_vesting),
				(team.clone(), SETM, 258, 1, 5_112_000, setm_team_vesting),

				(foundation.clone(), ETH, 258, 1, 5_112_000, serp_foundation_vesting),
				(team.clone(), ETH, 258, 1, 5_112_000, serp_team_vesting),

				(foundation.clone(), WBTC, 258, 1, 5_112_000, dnar_foundation_vesting),
				(team.clone(), WBTC, 258, 1, 5_112_000, dnar_team_vesting),

				(foundation.clone(), BNB, 258, 1, 5_112_000, help_foundation_vesting),
				(team.clone(), BNB, 258, 1, 5_112_000, help_team_vesting),

				(foundation.clone(), USDW, 258, 1, 5_112_000, setr_foundation_vesting),
				(team.clone(), USDW, 258, 1, 5_112_000, setr_team_vesting),

				(foundation.clone(), USDI, 258, 1, 5_112_000, setusd_foundation_vesting),
				(team.clone(), USDI, 258, 1, 5_112_000, setusd_team_vesting),
			]
		},
		cdp_treasury: CdpTreasuryConfig {
			expected_collateral_auction_size: vec![
				(SETM, 500 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(ETH, 300 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(WBTC, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(BNB, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
			],
		},
		cdp_engine: CdpEngineConfig {
			collaterals_params: vec![
				(
					SETM,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in USDI (cap)
				),
				(
					ETH,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in USDI (cap)
				),
				(
					WBTC,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in USDI (cap)
				),
				(
					BNB,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in USDI (cap)
				),
			],
		},
		dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		},
		orml_nft: OrmlNFTConfig { tokens: vec![] },
	}
}

/// Currencies Properties
pub fn newrome_properties() -> Properties {
	let mut properties = Map::new();
	let mut token_symbol: Vec<String> = vec![];
	let mut token_decimals: Vec<u32> = vec![];
	[SETM, ETH, WBTC, BNB, USDW, USDI].iter().for_each(|token| {
		token_symbol.push(token.symbol().unwrap().to_string());
		token_decimals.push(token.decimals().unwrap() as u32);
	});
	properties.insert("tokenSymbol".into(), token_symbol.into());
	properties.insert("tokenDecimals".into(), token_decimals.into());
	properties.insert("ss58Format".into(), SS58Prefix::get().into());

	properties
}


/// Predeployed contract addresses
pub fn evm_genesis() -> BTreeMap<H160, module_evm::GenesisAccount<Balance, Nonce>> {
	let existential_deposit = MaxNativeTokenExistentialDeposit::get();
	
	let contracts_json = &include_bytes!("../../../lib-serml/sevm/predeploy-contracts/resources/bytecodes.json")[..];
	let contracts: Vec<(String, String, String)> = serde_json::from_slice(contracts_json).unwrap();
	let mut accounts = BTreeMap::new();
	for (_, address, code_string) in contracts {
		let account = module_evm::GenesisAccount {
			nonce: 0,
			balance: existential_deposit,
			storage: Default::default(),
			code: if code_string.len().is_zero() {
				vec![]
			} else {
				Bytes::from_str(&code_string).unwrap().0
			},
		};
		let addr = H160::from_slice(
			from_hex(address.as_str())
				.expect("predeploy-contracts must specify address")
				.as_slice(),
		);
		accounts.insert(addr, account);
	}
	accounts
}
