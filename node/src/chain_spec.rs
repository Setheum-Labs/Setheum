// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
// ٱلَّذِينَ يَأْكُلُونَ ٱلرِّبَوٰا۟ لَا يَقُومُونَ إِلَّا كَمَا يَقُومُ ٱلَّذِى يَتَخَبَّطُهُ ٱلشَّيْطَـٰنُ مِنَ ٱلْمَسِّ ۚ ذَٰلِكَ بِأَنَّهُمْ قَالُوٓا۟ إِنَّمَا ٱلْبَيْعُ مِثْلُ ٱلرِّبَوٰا۟ ۗ وَأَحَلَّ ٱللَّهُ ٱلْبَيْعَ وَحَرَّمَ ٱلرِّبَوٰا۟ ۚ فَمَن جَآءَهُۥ مَوْعِظَةٌ مِّن رَّبِّهِۦ فَٱنتَهَىٰ فَلَهُۥ مَا سَلَفَ وَأَمْرُهُۥٓ إِلَى ٱللَّهِ ۖ وَمَنْ عَادَ فَأُو۟لَـٰٓئِكَ أَصْحَـٰبُ ٱلنَّارِ ۖ هُمْ فِيهَا خَـٰلِدُونَ

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
use setheum_runtime::{
	//
	AccountId, AirDropConfig, AirDropCurrencyId, CurrencyId,
	//
	BabeConfig, BalancesConfig, GenesisConfig, SystemConfig,
	SS58Prefix, opaque::SessionKeys, get_all_module_accounts,
	ImOnlineId, IndicesConfig, SessionConfig, StakingConfig,
	AuthorityDiscoveryId, EVMConfig, AuthorityDiscoveryConfig,
	StakerStatus,
	//
	SudoConfig,
	ShuraCouncilMembershipConfig,
	FinancialCouncilMembershipConfig,
	TechnicalCommitteeMembershipConfig,
	OperatorMembershipSetheumConfig,
	SerpTreasuryConfig,
	CdpTreasuryConfig,
	CdpEngineConfig,

	//
	DexConfig, EnabledTradingPairs,
	TokenSymbol, TokensConfig, OrmlNFTConfig,
	NativeTokenExistentialDeposit, MaxNativeTokenExistentialDeposit,
	//
	SETM, SERP, DNAR, SETR, SETUSD,
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

use setheum_primitives::{AccountPublic, Balance, Nonce, currency::TokenInfo};
use setheum_runtime::BABE_GENESIS_EPOCH_CONFIG;

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
	pub fork_blocks: sc_client_api::ForkBlocks<setheum_primitives::Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<setheum_primitives::Block>,
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
	let wasm_binary = setheum_runtime::WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
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
		Some(setheum_properties()),
		// Extensions
		Default::default(),
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = setheum_runtime::WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
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
		Some("setheum_local_testnet"),
		// Properties
		Some(setheum_properties()),
		// Extensions
		Default::default(),
	))
}

pub fn public_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = setheum_runtime::WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Setheum Testnet",
		// ID
		"setheum_testnet",
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
			// for i in 1; do for j in {stash, controller}; do subkey inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in grandpa; do subkey --ed25519 inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in babe; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in im_online; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in authority_discovery; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
			//
			// TODO: Update!
			vec![],
			// Sudo: TODO: Update to multisig account
			// hex!["0c994e7589709a85128a6695254af16227f7873816ae0269aa705861c315ba1e"].into(),
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// --------------------------------------------------------------------------------------------
			// Endowed accounts vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
			//
			vec![
				// // Foundation Faucet - TODO: Update to multisig
				// (hex!["9c48c0498bdf1d716f4544fc21f050963409f2db8154ba21e5233001202cbf08"].into()),
				// // Treasury Faucet- TODO: Update to `treasury_account`
				// (hex!["0c994e7589709a85128a6695254af16227f7873816ae0269aa705861c315ba1e"].into()),
				// // CashDropFund Faucet- TODO: Update to `into_account_id` from `cashdrop_pool_account`
				// (hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into()),
				// // PublicFund Faucet- TODO: Update
				// (hex!["5adebb35eb317412b58672db0434e4b112fcd27abaf28039f07c0db155b26650"].into()),
				// // Team and DEX Liquidity Offering Fund Faucet- TODO: Update to multisig
				// (hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into()),
				// // Advisors and Partners Fund Faucet- Labs - TODO: Update to multisig
				// (hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into()),
				// // Founder (Khalifa MBA) Faucet
				// hex!["6c1371ce4b06b8d191d6f552d716c00da31aca08a291ccbdeaf0f7aeae51201b"].into(),
			]
		),
		// Bootnodes - TODO: Update!
		vec![
			// "/dns/bootnode-t1.setheumscan.com/tcp/30334/p2p/12D3KooWKmFtS7BFtkkKWrP5ZcCpPFokmST2JFXFSsVBNeW5SXWg".parse().unwrap()
		],
		// Telemetry
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		// Protocol ID
		Some("setheum_testnet"),
		// Properties
		Some(setheum_properties()),
		// Extensions
		Default::default(),
	))
}


pub fn live_mainnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../resources/chain_spec_mainnet_raw.json")[..])
}

pub fn live_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../resources/chain_spec_testnet_raw.json")[..])
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = setheum_runtime::WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Setheum Mainnet",
		// ID
		"setheum_mainnet",
		ChainType::Live,
		move || mainnet_genesis(
			wasm_binary,
			// Initial authorities keys:
			// stash
			// controller
			// grandpa
			// babe
			// im-online
			// authority-discovery
			//
			// for i in 1; do for j in {stash, controller}; do subkey inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in grandpa; do subkey --ed25519 inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in babe; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in im_online; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
			// for i in 1; do for j in authority_discovery; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
			//
			// TODO: Update!
			vec![],
			// Sudo: TODO: Update to multisig account
			// hex!["0c994e7589709a85128a6695254af16227f7873816ae0269aa705861c315ba1e"].into(),
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// --------------------------------------------------------------------------------------------
			// Endowed accounts vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
			//
			vec![
				// // Foundation - TODO: Update to multisig
				// (hex!["9c48c0498bdf1d716f4544fc21f050963409f2db8154ba21e5233001202cbf08"].into()),
				// // Treasury - TODO: Update to `treasury_account`
				// (hex!["0c994e7589709a85128a6695254af16227f7873816ae0269aa705861c315ba1e"].into()),
				// // CashDropFund - TODO: Update to `into_account_id` from `cashdrop_pool_account`
				// (hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into()),
				// // PublicFund - TODO: Update
				// (hex!["5adebb35eb317412b58672db0434e4b112fcd27abaf28039f07c0db155b26650"].into()),
				// // Team and DEX Liquidity Offering Fund - TODO: Update to multisig
				// (hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into()),
				// // Advisors and Partners Fund - Labs - TODO: Update to multisig
				// (hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into()),
				// // Founder (Khalifa MBA) - TODO: Update
				// hex!["6c1371ce4b06b8d191d6f552d716c00da31aca08a291ccbdeaf0f7aeae51201b"].into(),
			],
			// ----------------------------------------------------------------------------------------
			// Mainnet Council Members and Initial Allocation Accounts vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
			//
			// Foundation - TODO: Update to multisig
			hex!["9c48c0498bdf1d716f4544fc21f050963409f2db8154ba21e5233001202cbf08"].into(),
			// Treasury - TODO: Update to `treasury_account`
			hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into(),
			// CashDropFund - TODO: Update to `into_account_id` from `cashdrop_pool_account`
			hex!["0c994e7589709a85128a6695254af16227f7873816ae0269aa705861c315ba1e"].into(),
			// PublicFundTreasury - TODO: Update to `into_account_id` from `PublicFundTreasuryModuleId`
			hex!["5adebb35eb317412b58672db0434e4b112fcd27abaf28039f07c0db155b26650"].into(),
			// Advisors and Partners Fund - Labs - TODO: Update to multisig
			hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into(),
			// Labs - Council Member - TODO: Update to multisig
			hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into()
		),
		// Bootnodes - TODO: Update!
		vec![
			// "/dns/bootnode.setheumscan.com/tcp/30333/p2p/12D3KooWFHSc9cUcyNtavUkLg4VBAeBnYNgy713BnovUa9WNY5pp".parse().unwrap(),
			// "/dns/bootnode.setheum.xyz/tcp/30333/p2p/12D3KooWAQqcXvcvt4eVEgogpDLAdGWgR5bY1drew44We6FfJAYq".parse().unwrap(),
			// "/dns/bootnode.setheum-chain.com/tcp/30333/p2p/12D3KooWCT7rnUmEK7anTp7svwr4GTs6k3XXnSjmgTcNvdzWzgWU".parse().unwrap(),
		],
		// Telemetry
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		// Protocol ID
		Some("setheum_mainnet"),
		// Properties
		Some(setheum_properties()),
		// Extensions
		Default::default(),
	))
}

fn dev_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {

	let existential_deposit = NativeTokenExistentialDeposit::get();

	let evm_genesis_accounts = evm_genesis();

	let initial_balance: u128 = 100_000_000_000_000_000_000_000; // 100,000 SETM
	let initial_staking: u128 = 100_000_000_000_000_000_000_000; // 100,000 SETM

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
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
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
		treasury: Default::default(), // Main Treasury (Setheum Treasury)
		tokens: TokensConfig {
			balances: endowed_accounts
				.iter()
				.flat_map(|x| {
					vec![
						(x.clone(), CurrencyId::Token(TokenSymbol::SETM), 100_000_000_000_000_000_000_000),  // 100,000 SETM
						(x.clone(), CurrencyId::Token(TokenSymbol::SERP), 100_000_000_000_000_000_000_000),  // 100,000 SERP
						(x.clone(), CurrencyId::Token(TokenSymbol::DNAR), 100_000_000_000_000_000_000_000),  // 100,000 DNAR
						(x.clone(), CurrencyId::Token(TokenSymbol::SETR), 100_000_000_000_000_000_000_000),  // 100,000 SETR
						(x.clone(), CurrencyId::Token(TokenSymbol::SETUSD), 100_000_000_000_000_000_000_000) // 100,000 SETUSD
					]
				})
				.collect(),
		},
		evm: EVMConfig {
			accounts: evm_genesis_accounts,
			treasury: root_key,
		},
		serp_treasury: SerpTreasuryConfig {
			stable_currency_inflation_rate: vec![
				(SETR, 50_000 * 1_000_000_000_000_000_000), 	// (currency_id, inflation rate of a setcurrency)
				(SETUSD, 100_000 * 1_000_000_000_000_000_000),  // (currency_id, inflation rate of a setcurrency)
			],
		},
		cdp_treasury: CdpTreasuryConfig {
			expected_collateral_auction_size: vec![
				(SETM, 500 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(SERP, 300 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(DNAR, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(SETR, 800 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
			],
		},
		cdp_engine: CdpEngineConfig {
			collaterals_params: vec![
				(
					SETM,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
				(
					SERP,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
				(
					DNAR,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
				(
					SETR,
					Some(FixedU128::saturating_from_rational(103, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(3, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(106, 100)), // required liquidation ratio
					33_000_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
			],
		},
		air_drop: AirDropConfig {
			airdrop_accounts: {
				let setter_airdrop_accounts_json = &include_bytes!("../../resources/testnet-airdrop-SETR.json")[..];
				let setter_airdrop_accounts: Vec<(AccountId, Balance)> =
					serde_json::from_slice(setter_airdrop_accounts_json).unwrap();
				let setdollar_airdrop_accounts_json = &include_bytes!("../../resources/testnet-airdrop-SETUSD.json")[..];
				let setdollar_airdrop_accounts: Vec<(AccountId, Balance)> =
					serde_json::from_slice(setdollar_airdrop_accounts_json).unwrap();

				setter_airdrop_accounts
					.iter()
					.map(|(account_id, setter_amount)| (account_id.clone(), AirDropCurrencyId::SETR, *setter_amount))
					.chain(
						setdollar_airdrop_accounts
							.iter()
							.map(|(account_id, setdollar_amount)| (account_id.clone(), AirDropCurrencyId::SETUSD, *setdollar_amount)),
					)
					.collect::<Vec<_>>()
			},
		},
		dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		},
		orml_nft: OrmlNFTConfig { tokens: vec![] },
	}
}

fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {

	let existential_deposit = NativeTokenExistentialDeposit::get();

	let evm_genesis_accounts = evm_genesis();

	let  initial_balance: u128 = 100_000 * 1_000_000_000_000_000_000;
	let  initial_staking: u128 = 100_000 * 1_000_000_000_000_000_000;

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), initial_staking + 1_000_000_000_000_000_000)) // bit more for fee
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
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
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
		treasury: Default::default(), // Main Treasury (Setheum Treasury)
		tokens: TokensConfig {
			balances: endowed_accounts
				.iter()
				.flat_map(|x| {
					vec![
						(x.clone(), CurrencyId::Token(TokenSymbol::SETM), 100_000 * 1_000_000_000_000_000_000),
						(x.clone(), CurrencyId::Token(TokenSymbol::SERP), 100_000 * 1_000_000_000_000_000_000),
						(x.clone(), CurrencyId::Token(TokenSymbol::DNAR), 100_000 * 1_000_000_000_000_000_000),
						(x.clone(), CurrencyId::Token(TokenSymbol::SETR), 100_000 * 1_000_000_000_000_000_000),
						(x.clone(), CurrencyId::Token(TokenSymbol::SETUSD), 100_000 * 1_000_000_000_000_000_000)
					]
				})
				.collect(),
		},
		evm: EVMConfig {
			accounts: evm_genesis_accounts,
			treasury: root_key,
		},
		serp_treasury: SerpTreasuryConfig {
			stable_currency_inflation_rate: vec![
				(SETR, 50_000 * 1_000_000_000_000_000_000), 	// (currency_id, inflation rate of a setcurrency)
				(SETUSD, 100_000 * 1_000_000_000_000_000_000),  // (currency_id, inflation rate of a setcurrency)
			],
		},
		cdp_treasury: CdpTreasuryConfig {
			expected_collateral_auction_size: vec![
				(SETM, 500 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(SERP, 300 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(DNAR, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(SETR, 800 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
			],
		},
		cdp_engine: CdpEngineConfig {
			collaterals_params: vec![
				(
					SETM,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
				(
					SERP,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
				(
					DNAR,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
				(
					SETR,
					Some(FixedU128::saturating_from_rational(103, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(3, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(106, 100)), // required liquidation ratio
					33_000_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
			],
		},
		air_drop: AirDropConfig {
			airdrop_accounts: {
				let setter_airdrop_accounts_json = &include_bytes!("../../resources/testnet-airdrop-SETR.json")[..];
				let setter_airdrop_accounts: Vec<(AccountId, Balance)> =
					serde_json::from_slice(setter_airdrop_accounts_json).unwrap();
				let setdollar_airdrop_accounts_json = &include_bytes!("../../resources/testnet-airdrop-SETUSD.json")[..];
				let setdollar_airdrop_accounts: Vec<(AccountId, Balance)> =
					serde_json::from_slice(setdollar_airdrop_accounts_json).unwrap();

				setter_airdrop_accounts
					.iter()
					.map(|(account_id, setter_amount)| (account_id.clone(), AirDropCurrencyId::SETR, *setter_amount))
					.chain(
						setdollar_airdrop_accounts
							.iter()
							.map(|(account_id, setdollar_amount)| (account_id.clone(), AirDropCurrencyId::SETUSD, *setdollar_amount)),
					)
					.collect::<Vec<_>>()
			},
		},
		dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		},
		orml_nft: OrmlNFTConfig { tokens: vec![] },
	}
}

fn mainnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	foundation: AccountId,
	treasury_fund: AccountId,
	cashdrop_fund: AccountId,
	spf_fund: AccountId,
	advisors_and_partners_fund: AccountId,
	labs: AccountId
) -> GenesisConfig {

	let evm_genesis_accounts = evm_genesis();

	let  setm_foundation_alloc: u128 = 626_600_000 * 1_000_000_000_000_000_000;
	let  setm_treasury_alloc: u128 = 313_300_000 * 1_000_000_000_000_000_000;
	let  setm_spf_alloc: u128 = 313_300_000 * 1_000_000_000_000_000_000;
	let  setm_team_alloc: u128 = 1_660_490_000 * 1_000_000_000_000_000_000;
	let  setm_advisors_n_partners_alloc: u128 = 250_640_000 * 1_000_000_000_000_000_000;

	let  serp_foundation_alloc: u128 = 51_600_000 * 1_000_000_000_000_000_000;
	let  serp_treasury_alloc: u128 = 25_800_000 * 1_000_000_000_000_000_000;
	let  serp_spf_alloc: u128 = 25_800_000 * 1_000_000_000_000_000_000;
	let  serp_team_alloc: u128 = 103_200_000 * 1_000_000_000_000_000_000;
	let  serp_advisors_n_partners_alloc: u128 = 20_640_000 * 1_000_000_000_000_000_000;

	let  dnar_foundation_alloc: u128 = 14_000_000 * 1_000_000_000_000_000_000;
	let  dnar_treasury_alloc: u128 = 7_000_000 * 1_000_000_000_000_000_000;
	let  dnar_spf_alloc: u128 = 7_000_000 * 1_000_000_000_000_000_000;
	let  dnar_team_alloc: u128 = 28_000_000 * 1_000_000_000_000_000_000;
	let  dnar_advisors_n_partners_alloc: u128 = 5_600_000 * 1_000_000_000_000_000_000;

	let  setr_foundation_alloc: u128 = 626_600_000 * 1_000_000_000_000_000_000;
	let  setr_treasury_alloc: u128 = 313_300_000 * 1_000_000_000_000_000_000;
	let  setr_cashdrop_alloc: u128 = 313_300_000 * 1_000_000_000_000_000_000;
	let  setr_spf_alloc: u128 = 313_300_000 * 1_000_000_000_000_000_000;
	let  setr_team_alloc: u128 = 830_245_000 * 1_000_000_000_000_000_000;
	let  setr_advisors_n_partners_alloc: u128 = 250_640_000 * 1_000_000_000_000_000_000;

	let  setusd_foundation_alloc: u128 = 1_253_200_000 * 1_000_000_000_000_000_000;
	let  setusd_treasury_alloc: u128 = 626_600_000 * 1_000_000_000_000_000_000;
	let  setusd_cashdrop_alloc: u128 = 626_600_000 * 1_000_000_000_000_000_000;
	let  setusd_spf_alloc: u128 = 626_600_000 * 1_000_000_000_000_000_000;
	let  setusd_team_alloc: u128 = 1_660_490_000 * 1_000_000_000_000_000_000;
	let  setusd_advisors_n_partners_alloc: u128 = 250_640_000 * 1_000_000_000_000_000_000;

	let  initial_balance: u128 = 10_000 * 1_000_000_000_000_000_000;
	let  initial_staking: u128 =   10_000 * 1_000_000_000_000_000_000;
	let existential_deposit = NativeTokenExistentialDeposit::get();

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), initial_staking))
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
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
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
		treasury: Default::default(), // Setheum Treasury
		tokens: TokensConfig {
			balances:
				vec![
					// SETM allocations
					(foundation.clone(), CurrencyId::Token(TokenSymbol::SETM), setm_foundation_alloc),
					(treasury_fund.clone(), CurrencyId::Token(TokenSymbol::SETM), setm_treasury_alloc),
					(spf_fund.clone(), CurrencyId::Token(TokenSymbol::SETM), setm_spf_alloc),
					(labs.clone(), CurrencyId::Token(TokenSymbol::SETM), setm_team_alloc),
					(advisors_and_partners_fund.clone(), CurrencyId::Token(TokenSymbol::SETM), setm_advisors_n_partners_alloc),
					// SERP allocations
					(foundation.clone(), CurrencyId::Token(TokenSymbol::SERP), serp_foundation_alloc),
					(treasury_fund.clone(), CurrencyId::Token(TokenSymbol::SERP), serp_treasury_alloc),
					(spf_fund.clone(), CurrencyId::Token(TokenSymbol::SERP), serp_spf_alloc),
					(labs.clone(), CurrencyId::Token(TokenSymbol::SERP), serp_team_alloc),
					(advisors_and_partners_fund.clone(), CurrencyId::Token(TokenSymbol::SERP), serp_advisors_n_partners_alloc),
					// DNAR allocations
					(foundation.clone(), CurrencyId::Token(TokenSymbol::DNAR), dnar_foundation_alloc),
					(treasury_fund.clone(), CurrencyId::Token(TokenSymbol::DNAR), dnar_treasury_alloc),
					(spf_fund.clone(), CurrencyId::Token(TokenSymbol::DNAR), dnar_spf_alloc),
					(labs.clone(), CurrencyId::Token(TokenSymbol::DNAR), dnar_team_alloc),
					(advisors_and_partners_fund.clone(), CurrencyId::Token(TokenSymbol::DNAR), dnar_advisors_n_partners_alloc),
					// SETR allocations
					(foundation.clone(), CurrencyId::Token(TokenSymbol::SETR), setr_foundation_alloc),
					(treasury_fund.clone(), CurrencyId::Token(TokenSymbol::SETR), setr_treasury_alloc),
					(cashdrop_fund.clone(), CurrencyId::Token(TokenSymbol::SETR), setr_cashdrop_alloc),
					(spf_fund.clone(), CurrencyId::Token(TokenSymbol::SETR), setr_spf_alloc),
					(labs.clone(), CurrencyId::Token(TokenSymbol::SETR), setr_team_alloc),
					(advisors_and_partners_fund.clone(), CurrencyId::Token(TokenSymbol::SETR), setr_advisors_n_partners_alloc),
					// SETUSD allocations
					(foundation.clone(), CurrencyId::Token(TokenSymbol::SETUSD), setusd_foundation_alloc),
					(treasury_fund.clone(), CurrencyId::Token(TokenSymbol::SETUSD), setusd_treasury_alloc),
					(cashdrop_fund.clone(), CurrencyId::Token(TokenSymbol::SETUSD), setusd_cashdrop_alloc),
					(spf_fund.clone(), CurrencyId::Token(TokenSymbol::SETUSD), setusd_spf_alloc),
					(labs.clone(), CurrencyId::Token(TokenSymbol::SETUSD), setusd_team_alloc),
					(advisors_and_partners_fund.clone(), CurrencyId::Token(TokenSymbol::SETUSD), setusd_advisors_n_partners_alloc),
				]
		},
		evm: EVMConfig {
			accounts: evm_genesis_accounts,
			treasury: root_key,
		},
		serp_treasury: SerpTreasuryConfig {
			stable_currency_inflation_rate: vec![
				(SETR, 50_000 * 1_000_000_000_000_000_000), 	// (currency_id, inflation rate of a setcurrency)
				(SETUSD, 100_000 * 1_000_000_000_000_000_000),  // (currency_id, inflation rate of a setcurrency)
			],
		},
		cdp_treasury: CdpTreasuryConfig {
			expected_collateral_auction_size: vec![
				(SETM, 500 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(SERP, 300 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(DNAR, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(SETR, 800 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
			],
		},
		cdp_engine: CdpEngineConfig {
			collaterals_params: vec![
				(
					SETM,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
				(
					SERP,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
				(
					DNAR,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
				(
					SETR,
					Some(FixedU128::saturating_from_rational(103, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(3, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(106, 100)), // required liquidation ratio
					33_000_000 * 1_000_000_000_000_000_000,              // maximum debit value in SETUSD (cap)
				),
			],
		},
		air_drop: AirDropConfig {
			airdrop_accounts: {
				let setter_airdrop_accounts_json = &include_bytes!("../../resources/mainnet-airdrop-SETR.json")[..];
				let setter_airdrop_accounts: Vec<(AccountId, Balance)> =
					serde_json::from_slice(setter_airdrop_accounts_json).unwrap();
				let setdollar_airdrop_accounts_json = &include_bytes!("../../resources/mainnet-airdrop-SETUSD.json")[..];
				let setdollar_airdrop_accounts: Vec<(AccountId, Balance)> =
					serde_json::from_slice(setdollar_airdrop_accounts_json).unwrap();

				setter_airdrop_accounts
					.iter()
					.map(|(account_id, setter_amount)| (account_id.clone(), AirDropCurrencyId::SETR, *setter_amount))
					.chain(
						setdollar_airdrop_accounts
							.iter()
							.map(|(account_id, setdollar_amount)| (account_id.clone(), AirDropCurrencyId::SETUSD, *setdollar_amount)),
					)
					.collect::<Vec<_>>()
			},
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
pub fn setheum_properties() -> Properties {
	let mut properties = Map::new();
	let mut token_symbol: Vec<String> = vec![];
	let mut token_decimals: Vec<u32> = vec![];
	[SETM, SERP, DNAR, SETR, SETUSD].iter().for_each(|token| {
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
	
	let contracts_json = &include_bytes!("../../predeploy-contracts/resources/bytecodes.json")[..];
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
