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
use setheum_runtime::{
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
	SerpTreasuryConfig,
	CdpTreasuryConfig,
	CdpEngineConfig,

	//
	DexConfig, EnabledTradingPairs,
	TokensConfig, OrmlNFTConfig,
	NativeTokenExistentialDeposit, MaxNativeTokenExistentialDeposit,
	//
	SETM, SERP, DNAR, HELP, SETR, SETUSD,
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

use setheum_primitives::{AccountPublic, Balance, Nonce, currency::TokenInfo, TradingPair};
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
				// Setheum Public Fund (SPF): VQiLsC6xs5xSG7jFUbcRCjKPZqnacJmrNANovRHzbtgThHzhy
				(hex!["78d105e22be9735d200591ebe506fbc0d0be3f18afa5f5b2fbdb370ee4c2fd47"].into(), 313_300_000 as u128),
				// Team and DEX Liquidity Offering Fund: VQgPxsHbvGdXC7HhUvYvPifu1SyAuRnUhbMw4hAaTm9fwvkkz
				(hex!["22b565e2303579c0d50884a3524c32ed12c8b91a8621dd72270b8fd17d20d009"].into(), 1_566_500_000 as u128),
				// Advisors and Partners Fund: VQgfLtTS8oZCreyX3FzHuaAbUovtbcuSFLnUFS3tkRvwWGkbD
				(hex!["2e70349d7140ec49b7cf1ae03b6ae3405103dab86c5a463ceef77ffb4a769868"].into(), 156_650_000 as u128),
			],
			// Foundation & Airdrop: VQh5AghroqszPazCMEdpN8QkAMvXCTkXbANx6GSqFz2kTebuv
			hex!["409bc00c7f4d8cf046c1eb363022eec1103e70ae180cba92056452315837c71a"].into(),
			// Setheum Public Fund (SPF): VQiLsC6xs5xSG7jFUbcRCjKPZqnacJmrNANovRHzbtgThHzhy
			hex!["78d105e22be9735d200591ebe506fbc0d0be3f18afa5f5b2fbdb370ee4c2fd47"].into(),
			// Team and DEX Liquidity Offering Fund: VQgPxsHbvGdXC7HhUvYvPifu1SyAuRnUhbMw4hAaTm9fwvkkz
			hex!["22b565e2303579c0d50884a3524c32ed12c8b91a8621dd72270b8fd17d20d009"].into(),
			// Advisors and Partners Fund: VQgfLtTS8oZCreyX3FzHuaAbUovtbcuSFLnUFS3tkRvwWGkbD
			hex!["2e70349d7140ec49b7cf1ae03b6ae3405103dab86c5a463ceef77ffb4a769868"].into(),
		),
		// Bootnodes - TODO: Update!
		vec![],
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

	let initial_balance: u128 = 10_000 * 1_000_000_000_000_000_000;	// 1,000,000 SETM/SERP/DNAR/HELP/SETR/SETUSD
	let initial_staking: u128 = 2_000 * 1_000_000_000_000_000_000; 	// 258,000 SETM/SERP/DNAR/HELP/SETR/SETUSD

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
		treasury: Default::default(), // Main Treasury (Setheum Treasury)
		tokens: TokensConfig {
			balances: endowed_accounts
				.iter()
				.flat_map(|x| vec![
					(x.clone(), SERP, initial_balance),
					(x.clone(), DNAR, initial_balance),
					(x.clone(), HELP, initial_balance),
					(x.clone(), SETR, initial_balance),
					(x.clone(), SETUSD, initial_balance),
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
				(x.clone(), SERP, 10, 1, 3600, 100_000_000_000_000_000),
				(x.clone(), DNAR, 10, 1, 3600, 100_000_000_000_000_000),
				(x.clone(), HELP, 10, 1, 3600, 100_000_000_000_000_000),
				(x.clone(), SETR, 10, 1, 3600, 100_000_000_000_000_000),
				(x.clone(), SETUSD, 10, 1, 3600, 100_000_000_000_000_000),
				])
			.collect(),
		},
		serp_treasury: SerpTreasuryConfig {
			stable_currency_inflation_rate: vec![
				(SETR, 100_000_000_000_000_000_000), 	// (currency_id, inflation rate of a setcurrency)
				(SETUSD, 10_000_000_000_000_000_000),	// (currency_id, inflation rate of a setcurrency)
			],
			stable_currency_cashdrop: vec![
				(SETR,  initial_balance), 	// (currency_id, cashdrop pool balance of a setcurrency)
				(SETUSD,  initial_balance),  // (currency_id, cashdrop pool balance of a setcurrency)
			],
		},
		cdp_treasury: CdpTreasuryConfig {
			expected_collateral_auction_size: vec![
				(SETM, 500 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(SERP, 300 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(DNAR, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(HELP, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
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
					HELP,
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
		treasury: Default::default(), // Main Treasury (Setheum Treasury)
		tokens: TokensConfig {
			balances: vec![
				(foundation.clone(), SERP, serp_foundation_alloc + serp_airdrops_alloc),
				(team.clone(), SERP, serp_team_alloc),
				(foundation.clone(), DNAR, dnar_foundation_alloc + dnar_airdrop_alloc),
				(team.clone(), DNAR, dnar_team_alloc),
				(foundation.clone(), HELP, help_foundation_alloc + help_airdrop_alloc),
				(team.clone(), HELP, help_team_alloc),
				(foundation.clone(), SETR, setr_foundation_alloc + setr_airdrop_alloc),
				(team.clone(), SETR, setr_team_alloc),
				(foundation.clone(), SETUSD, setusd_foundation_alloc + setusd_airdrop_alloc),
				(team.clone(), SETUSD, setusd_team_alloc),
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

				(foundation.clone(), SERP, 258, 1, 5_112_000, serp_foundation_vesting),
				(team.clone(), SERP, 258, 1, 5_112_000, serp_team_vesting),

				(foundation.clone(), DNAR, 258, 1, 5_112_000, dnar_foundation_vesting),
				(team.clone(), DNAR, 258, 1, 5_112_000, dnar_team_vesting),

				(foundation.clone(), HELP, 258, 1, 5_112_000, help_foundation_vesting),
				(team.clone(), HELP, 258, 1, 5_112_000, help_team_vesting),

				(foundation.clone(), SETR, 258, 1, 5_112_000, setr_foundation_vesting),
				(team.clone(), SETR, 258, 1, 5_112_000, setr_team_vesting),

				(foundation.clone(), SETUSD, 258, 1, 5_112_000, setusd_foundation_vesting),
				(team.clone(), SETUSD, 258, 1, 5_112_000, setusd_team_vesting),
			]
		},
		serp_treasury: SerpTreasuryConfig {
			stable_currency_inflation_rate: vec![
				(SETR, 0), 	// (currency_id, inflation rate of a setcurrency) to be set on-chain;
				(SETUSD, 0),	// (currency_id, inflation rate of a setcurrency) to be set on-chain;
			],
			stable_currency_cashdrop: vec![
				(SETR,  setr_cashdrop_alloc), 	// (currency_id, cashdrop pool balance of a setcurrency)
				(SETUSD,  setusd_cashdrop_alloc),  // (currency_id, cashdrop pool balance of a setcurrency)
			],
		},
		cdp_treasury: CdpTreasuryConfig {
			expected_collateral_auction_size: vec![
				(SETM, 500 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(SERP, 300 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(DNAR, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(HELP, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
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
					HELP,
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
	endowed_accounts: Vec<(AccountId, Balance)>,
	foundation: AccountId,
	spf: AccountId,
	team: AccountId,
	advisors_n_partners: AccountId,
) -> GenesisConfig {
	// Allocations Endowment
	let  serp_foundation_alloc: u128 = 51_600_000 * 1_000_000_000_000_000_000;
	let  serp_spf_alloc: u128 = 25_800_000 * 1_000_000_000_000_000_000;
	let  serp_team_alloc: u128 = 154_800_000 * 1_000_000_000_000_000_000;
	let  serp_advisors_n_partners_alloc: u128 = 12_900_000 * 1_000_000_000_000_000_000;
	let  serp_airdrops_alloc: u128 = 12_900_000 * 1_000_000_000_000_000_000;
	
	let  dnar_foundation_alloc: u128 = 14_000_000 * 1_000_000_000_000_000_000;
	let  dnar_spf_alloc: u128 = 7_000_000 * 1_000_000_000_000_000_000;
	let  dnar_team_alloc: u128 = 42_000_000 * 1_000_000_000_000_000_000;
	let  dnar_advisors_n_partners_alloc: u128 = 3_500_000 * 1_000_000_000_000_000_000;
	let  dnar_airdrop_alloc: u128 = 3_500_000 * 1_000_000_000_000_000_000;
	
	let  help_foundation_alloc: u128 = 140_000_000 * 1_000_000_000_000_000_000;
	let  help_spf_alloc: u128 = 70_000_000 * 1_000_000_000_000_000_000;
	let  help_team_alloc: u128 = 420_000_000 * 1_000_000_000_000_000_000;
	let  help_advisors_n_partners_alloc: u128 = 35_000_000 * 1_000_000_000_000_000_000;
	let  help_airdrop_alloc: u128 = 35_000_000 * 1_000_000_000_000_000_000;
	
	let  setr_foundation_alloc: u128 = 200_000_000 * 1_000_000_000_000_000_000;
	let  setr_spf_alloc: u128 = 200_000_000 * 1_000_000_000_000_000_000;
	let  setr_team_alloc: u128 = 1_000_000_000 * 1_000_000_000_000_000_000;
	let  setr_advisors_n_partners_alloc: u128 = 200_000_000 * 1_000_000_000_000_000_000;
	let  setr_cashdrop_alloc: u128 = 200_000_000 * 1_000_000_000_000_000_000;
	let  setr_airdrop_alloc: u128 = 200_000_000 * 1_000_000_000_000_000_000;
	
	let  setusd_foundation_alloc: u128 = 31_330_000 * 1_000_000_000_000_000_000;
	let  setusd_spf_alloc: u128 = 31_330_000 * 1_000_000_000_000_000_000;
	let  setusd_team_alloc: u128 = 156_650_000 * 1_000_000_000_000_000_000;
	let  setusd_advisors_n_partners_alloc: u128 = 31_330_000 * 1_000_000_000_000_000_000;
	let  setusd_cashdrop_alloc: u128 = 31_330_000 * 1_000_000_000_000_000_000;
	let  setusd_airdrop_alloc: u128 = 31_330_000 * 1_000_000_000_000_000_000;
	
	// Allocations Vesting - per_period
	let  setm_foundation_vesting: u128 = 2_664_659_454_310_403_000; // 2.664_659_454_310_403_000
	let  setm_spf_vesting: u128 = 16_316_348_637_613_530_00; // 16.316_348_637_613_530_00
	let  setm_team_vesting: u128 = 3_996_989_181_465_605_000; // 3.996_989_181_465_605_000
	let  setm_advisors_n_partners_vesting: u128 = 2_188_827_408_897_831_000; // 2.188_827_408_897_831_000
	
	let  serp_foundation_vesting: u128 = 276_254_015_319_990_000; // 0.276_254_015_319_990_000
	let  serp_spf_vesting: u128 = 374_916_163_648_558_000; // 0.374_916_163_648_558_000
	let  serp_team_vesting: u128 = 828_762_045_959_970_000; // 0.828_762_045_959_970_000
	let  serp_advisors_n_partners_vesting: u128 = 262_441_314_553_991_000; // 0.262_441_314_553_991_000
	
	let  dnar_foundation_vesting: u128 = 74_952_639_815_501_000; // 0.074_952_639_815_501_000
	let  dnar_spf_vesting: u128 = 101_721_439_749_609_000; // 0.101_721_439_749_609_000
	let  dnar_team_vesting: u128 = 224_857_919_446_504_000; // 0.224_857_919_446_504_000
	let  dnar_advisors_n_partners_vesting: u128 = 71_205_007_824_726_000; // 0.071_205_007_824_726_000
	
	let  help_foundation_vesting: u128 = 749_526_398_155_012_000; // 0.749_526_398_155_012_000
	let  help_spf_vesting: u128 = 1_017_214_397_496_088_000; // 1.017_214_397_496_088_000
	let  help_team_vesting: u128 = 2_248_579_194_465_036_000; // 2.248_579_194_465_036_000
	let  help_advisors_n_partners_vesting: u128 = 712_050_078_247_261_000; // 0.712_050_078_247_261_000
	
	let  setr_foundation_vesting: u128 = 3_912_363_067_292_645_000; // 3.912_363_067_292_645_000
	let  setr_spf_vesting: u128 = 3_912_363_067_292_645_000; // 3.912_363_067_292_645_000
	let  setr_team_vesting: u128 = 11_737_089_201_877_930_000; // 11.737_089_201_877_930_000
	let  setr_advisors_n_partners_vesting: u128 = 3_912_363_067_292_645_000; // 3.912_363_067_292_645_000
	
	let  setusd_foundation_vesting: u128 = 766_089_593_114_241_000; // 0.766_089_593_114_241_000
	let  setusd_spf_vesting: u128 = 766_089_593_114_241_000; // 0.766_089_593_114_241_000
	let  setusd_team_vesting: u128 = 2_298_268_779_342_723_000; // 2.298_268_779_342_723_000
	let  setusd_advisors_n_partners_vesting: u128 = 766_089_593_114_241_000; // 0.766_089_593_114_241_000

	let evm_genesis_accounts = evm_genesis();

	let  initial_staking: u128 = 100_000 * 1_000_000_000_000_000_000;
	let existential_deposit = NativeTokenExistentialDeposit::get();

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
				(root_key.clone()),
			],
			phantom: Default::default(),
		},
		financial_council: Default::default(),
		financial_council_membership: FinancialCouncilMembershipConfig {
			members: vec![
				(root_key.clone()),
			],
			phantom: Default::default(),
		},
		technical_committee: Default::default(),
		technical_committee_membership: TechnicalCommitteeMembershipConfig {
			members: vec![
				(root_key.clone()),
			],
			phantom: Default::default(),
		},
		operator_membership_setheum: OperatorMembershipSetheumConfig {
			members: vec![
				(root_key.clone()),
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
		treasury: Default::default(), // Setheum Treasury
		tokens: TokensConfig {
			balances: vec![
				(foundation.clone(), SERP, serp_foundation_alloc + serp_airdrops_alloc),
				(spf.clone(), SERP, serp_spf_alloc),
				(team.clone(), SERP, serp_team_alloc),
				(advisors_n_partners.clone(), SERP, serp_advisors_n_partners_alloc),

				(foundation.clone(), DNAR, dnar_foundation_alloc + dnar_airdrop_alloc),
				(spf.clone(), DNAR, dnar_spf_alloc),
				(team.clone(), DNAR, dnar_team_alloc),
				(advisors_n_partners.clone(), DNAR, dnar_advisors_n_partners_alloc),

				(foundation.clone(), HELP, help_foundation_alloc + help_airdrop_alloc),
				(spf.clone(), HELP, help_spf_alloc),
				(team.clone(), HELP, help_team_alloc),
				(advisors_n_partners.clone(), HELP, help_advisors_n_partners_alloc),

				(foundation.clone(), SETR, setr_foundation_alloc + setr_airdrop_alloc),
				(spf.clone(), SETR, setr_spf_alloc),
				(team.clone(), SETR, setr_team_alloc),
				(advisors_n_partners.clone(), SETR, setr_advisors_n_partners_alloc),

				(foundation.clone(), SETUSD, setusd_foundation_alloc + setusd_airdrop_alloc),
				(spf.clone(), SETUSD, setusd_spf_alloc),
				(team.clone(), SETUSD, setusd_team_alloc),
				(advisors_n_partners.clone(), SETUSD, setusd_advisors_n_partners_alloc),
			]
		},
		evm: EVMConfig {
			accounts: evm_genesis_accounts,
			treasury: root_key,
		},
		vesting: VestingConfig {
			vesting: vec![
				(foundation.clone(), SETM, 313, 1, 117_576_000, setm_foundation_vesting),
				(spf.clone(), SETM, 313, 1, 96_008_000, setm_spf_vesting),
				(team.clone(), SETM, 313, 1, 117_576_000, setm_team_vesting),
				(advisors_n_partners.clone(), SETM, 313, 1, 35_784_000, setm_advisors_n_partners_vesting),
				
				(foundation.clone(), SERP, 313, 1, 97_128_000, serp_foundation_vesting),
				(spf.clone(), SERP, 313, 1, 35_784_000, serp_spf_vesting),
				(team.clone(), SERP, 313, 1, 97_128_000, serp_team_vesting),
				(advisors_n_partners.clone(), SERP, 313, 1, 25_560_000, serp_advisors_n_partners_vesting),
				
				(foundation.clone(), DNAR, 313, 1, 97_128_000, dnar_foundation_vesting),
				(spf.clone(), DNAR, 313, 1, 35_784_000, dnar_spf_vesting),
				(team.clone(), DNAR, 313, 1, 97_128_000, dnar_team_vesting),
				(advisors_n_partners.clone(), DNAR, 313, 1, 25_560_000, dnar_advisors_n_partners_vesting),
				
				(foundation.clone(), HELP, 313, 1, 97_128_000, help_foundation_vesting),
				(spf.clone(), HELP, 313, 1, 35_784_000, help_spf_vesting),
				(team.clone(), HELP, 313, 1, 97_128_000, help_team_vesting),
				(advisors_n_partners.clone(), HELP, 313, 1, 25_560_000, help_advisors_n_partners_vesting),
				
				(foundation.clone(), SETR, 313, 1, 15_336_000, setr_foundation_vesting),
				(spf.clone(), SETR, 313, 1, 15_336_000, setr_spf_vesting),
				(team.clone(), SETR, 313, 1, 15_336_000, setr_team_vesting),
				(advisors_n_partners.clone(), SETR, 313, 1, 15_336_000, setr_advisors_n_partners_vesting),
				
				(foundation.clone(), SETUSD, 313, 1, 10_224_000, setusd_foundation_vesting),
				(spf.clone(), SETUSD, 313, 1, 10_224_000, setusd_spf_vesting),
				(team.clone(), SETUSD, 313, 1, 10_224_000, setusd_team_vesting),
				(advisors_n_partners.clone(), SETUSD, 313, 1, 10_224_000, setusd_advisors_n_partners_vesting),
			]
		},
		serp_treasury: SerpTreasuryConfig {
			stable_currency_inflation_rate: vec![
				(SETR, 0), 	// (currency_id, inflation rate of a setcurrency) to be set on-chain;
				(SETUSD, 0),	// (currency_id, inflation rate of a setcurrency) to be set on-chain;
			],
			stable_currency_cashdrop: vec![
				(SETR,  setr_cashdrop_alloc), 	// (currency_id, cashdrop pool balance of a setcurrency)
				(SETUSD,  setusd_cashdrop_alloc),  // (currency_id, cashdrop pool balance of a setcurrency)
			],
		},
		cdp_treasury: CdpTreasuryConfig {
			expected_collateral_auction_size: vec![
				(SETM, 500 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(SERP, 300 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(DNAR, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
				(HELP, 100 * 1_000_000_000_000_000_000), 		// (currency_id, max size of a collateral auction)
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
					HELP,
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
	[SETM, SERP, DNAR, HELP, SETR, SETUSD].iter().for_each(|token| {
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
	
	let contracts_json = &include_bytes!("../../lib-serml/sevm/predeploy-contracts/resources/bytecodes.json")[..];
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
