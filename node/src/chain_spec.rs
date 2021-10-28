use sp_core::{Pair, Public, sr25519, H160, Bytes};
use setheum_runtime::{
	AccountId, AirDropConfig, AirDropCurrencyId, CurrencyId, BabeConfig, Balance, BalancesConfig, BlockNumber, GenesisConfig, GrandpaConfig,
	SudoConfig, SystemConfig, IndicesConfig, EvmConfig, TradingPair, EnabledTradingPairs, StakingConfig, SessionConfig, AuthorityDiscoveryConfig,
	WASM_BINARY, dollar, get_all_module_accounts, SS58Prefix, TokenSymbol, TokensConfig, StakerStatus, ImOnlineId, Period, AuthorityDiscoveryId,
	CdpEngineConfig, CdpTreasuryConfig, SystemConfig, NativeTokenExistentialDeposit, opaque::SessionKeys, RenVmBridgeConfig, SessionManagerConfig,
	DexConfig, ShuraCouncilMembershipConfig, SudoConfig, OrmlNFTConfig, FinancialCouncilMembershipConfig, PublicFundCouncilMembershipConfig,
	TechnicalCommitteeMembershipConfig, OperatorMembershipSetheumConfig, SETM, SERP, DNAR, SETR, SETUSD, RENBTC,
};
use module_staking::Forcing;
use primitives::TokenInfo;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;

use sp_std::{collections::btree_map::BTreeMap, str::FromStr};
use sc_chain_spec::ChainSpecExtension;

use serde::{Deserialize, Serialize};

use hex_literal::hex;
use sp_core::{crypto::UncheckedInto, bytes::from_hex};

use setheum_primitives::{AccountPublic, Balance, Nonce};

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
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || testnet_genesis(
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
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			],
			// Multicurrency Pre-funded accounts
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
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
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"setheum_local_testnet",
		ChainType::Local,
		move || testnet_genesis(
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
			],
			// Multicurrency Pre-funded accounts
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
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
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
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
			vec![
				//Auth1 Validator
				(
					// stash 
					hex!["6c08c1f8e0cf1e200b24b43fca4c4e407b963b6b1e459d1aeff80c566a1da469"].into(),
					// controller
					hex!["864eff3160ff8609c030316867630850a9d6e35c47d3efe54de44264fef7665e"].into(),
					// grandpa 
					hex!["dc41d9325da71d90806d727b826d125cd523da28eb39ab048ab983d7bb74fb32"].unchecked_into(),
					// babe 
					hex!["8a688a748fd39bedaa507c942600c40478c2082dee17b8263613fc3c086b0c53"].unchecked_into(),
					// im-online
					hex!["3a4e80c48718f72326b49c4ae80199d35285643751e75a743f30b7561b538676"].unchecked_into(),
					// authority-discovery
					hex!["68d39d0d386ed4e9dd7e280d62e7dc9cf61dc508ef25efb74b6d740fa4dde463"].unchecked_into(),
				),
				//Auth1 Validator
				(
					// stash 
					hex!["6c08c1f8e0cf1e200b24b43fca4c4e407b963b6b1e459d1aeff80c566a1da469"].into(),
					// controller
					hex!["864eff3160ff8609c030316867630850a9d6e35c47d3efe54de44264fef7665e"].into(),
					// grandpa 
					hex!["dc41d9325da71d90806d727b826d125cd523da28eb39ab048ab983d7bb74fb32"].unchecked_into(),
					// babe 
					hex!["8a688a748fd39bedaa507c942600c40478c2082dee17b8263613fc3c086b0c53"].unchecked_into(),
					// im-online
					hex!["3a4e80c48718f72326b49c4ae80199d35285643751e75a743f30b7561b538676"].unchecked_into(),
					// authority-discovery
					hex!["68d39d0d386ed4e9dd7e280d62e7dc9cf61dc508ef25efb74b6d740fa4dde463"].unchecked_into(),
				),
			],
			// Sudo: TODO: Update to multisig
			hex!["9c48c0498bdf1d716f4544fc21f050963409f2db8154ba21e5233001202cbf08"].into(),
			// --------------------------------------------------------------------------------------------
			// Endowed accounts vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
			//
			vec![
				// Foundation Faucet - TODO: Update to multisig
				(hex!["9c48c0498bdf1d716f4544fc21f050963409f2db8154ba21e5233001202cbf08"].into()),
				// Treasury Faucet- TODO: Update to `treasury_account`
				(hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into()),
				// CashDropFund Faucet- TODO: Update to `into_account_id` from `cashdrop_pool_account`
				(hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into()),
				// PublicFundTreasury Faucet- TODO: Update to `into_account_id` from `PublicFundTreasuryModuleId`
				(hex!["5adebb35eb317412b58672db0434e4b112fcd27abaf28039f07c0db155b26650"].into()),
				// Team and DEX Liquidity Offering Fund Faucet- TODO: Update to multisig
				(hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into()),
				// Advisors and Partners Fund Faucet- Labs - TODO: Update to multisig
				(hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into()),
				// Labs Faucet- Shura Council Member - TODO: Update to multisig
				hex!["6c1371ce4b06b8d191d6f552d716c00da31aca08a291ccbdeaf0f7aeae51201b"].into(),
				// Muhammad-Jibril Bin Abdullah-Bashir Al-Sharif. (Khalifa MBA) Faucet
				hex!["6c1371ce4b06b8d191d6f552d716c00da31aca08a291ccbdeaf0f7aeae51201b"].into(),
			],
			// ----------------------------------------------------------------------------------------
			// Testnet Council Members vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
			//
			// Treasury - TODO: Update to `treasury_account`
			hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into(),
			// CashDropFund - TODO: Update to `cashdrop_pool_account`
			hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into(),
			// PublicFundTreasury - TODO: Update to `into_account_id` from `PublicFundTreasuryModuleId`
			hex!["5adebb35eb317412b58672db0434e4b112fcd27abaf28039f07c0db155b26650"].into(),
			// Team and DEX Liquidity Offering Fund
			hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into(),
			// Advisors and Partners Fund
			hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into(),
			// Labs - Shura Council Member
			hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into(),
			// Muhammad-Jibril Bin Abdullah-Bashir Al-Sharif. (Khalifa MBA) - Shura Council Member
			hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into(),
		),
		// Bootnodes
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
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
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
			vec![
				//Auth1 Validator
				(
					// stash 
					hex!["6c08c1f8e0cf1e200b24b43fca4c4e407b963b6b1e459d1aeff80c566a1da469"].into(),
					// controller
					hex!["864eff3160ff8609c030316867630850a9d6e35c47d3efe54de44264fef7665e"].into(),
					// grandpa 
					hex!["dc41d9325da71d90806d727b826d125cd523da28eb39ab048ab983d7bb74fb32"].unchecked_into(),
					// babe 
					hex!["8a688a748fd39bedaa507c942600c40478c2082dee17b8263613fc3c086b0c53"].unchecked_into(),
					// im-online
					hex!["3a4e80c48718f72326b49c4ae80199d35285643751e75a743f30b7561b538676"].unchecked_into(),
					// authority-discovery
					hex!["68d39d0d386ed4e9dd7e280d62e7dc9cf61dc508ef25efb74b6d740fa4dde463"].unchecked_into(),
				),
				//Auth1 Validator
				(
					// stash 
					hex!["6c08c1f8e0cf1e200b24b43fca4c4e407b963b6b1e459d1aeff80c566a1da469"].into(),
					// controller
					hex!["864eff3160ff8609c030316867630850a9d6e35c47d3efe54de44264fef7665e"].into(),
					// grandpa 
					hex!["dc41d9325da71d90806d727b826d125cd523da28eb39ab048ab983d7bb74fb32"].unchecked_into(),
					// babe 
					hex!["8a688a748fd39bedaa507c942600c40478c2082dee17b8263613fc3c086b0c53"].unchecked_into(),
					// im-online
					hex!["3a4e80c48718f72326b49c4ae80199d35285643751e75a743f30b7561b538676"].unchecked_into(),
					// authority-discovery
					hex!["68d39d0d386ed4e9dd7e280d62e7dc9cf61dc508ef25efb74b6d740fa4dde463"].unchecked_into(),
				),
			],
			// Sudo: TODO: Update to multisig
			hex!["9c48c0498bdf1d716f4544fc21f050963409f2db8154ba21e5233001202cbf08"].into(),
			// --------------------------------------------------------------------------------------------
			// Endowed accounts vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
			//
			vec![
				// Foundation - TODO: Update to multisig
				(hex!["9c48c0498bdf1d716f4544fc21f050963409f2db8154ba21e5233001202cbf08"].into()),
				// Treasury - TODO: Update to `treasury_account`
				(hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into()),
				// CashDropFund - TODO: Update to `into_account_id` from `cashdrop_pool_account`
				(hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into()),
				// PublicFundTreasury - TODO: Update to `into_account_id` from `PublicFundTreasuryModuleId`
				(hex!["5adebb35eb317412b58672db0434e4b112fcd27abaf28039f07c0db155b26650"].into()),
				// Team and DEX Liquidity Offering Fund - TODO: Update to multisig
				(hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into()),
				// Advisors and Partners Fund - Labs - TODO: Update to multisig
				(hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into()),
				// Labs - Shura Council Member - TODO: Update to multisig
				(hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into()),
				// Muhammad-Jibril B.A. (Khalifa) - Shura Council Member
				(hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into()),
			],
			// ----------------------------------------------------------------------------------------
			// Council Members vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
			//
			// Treasury - TODO: Update to `treasury_account`
			hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into(),
			// CashDropFund - TODO: Update to `cashdrop_pool_account`
			hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into(),
			// PublicFundTreasury - TODO: Update to `into_account_id` from `PublicFundTreasuryModuleId`
			hex!["5adebb35eb317412b58672db0434e4b112fcd27abaf28039f07c0db155b26650"].into(),
			// Team and DEX Liquidity Offering Fund
			hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into(),
			// Advisors and Partners Fund
			hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into(),
			// Labs - Shura Council Member
			hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into(),
			// Muhammad-Jibril Bin Abdullah-Bashir Al-Sharif. (Khalifa MBA) - Shura Council Member
			hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into(),
		),
		// Bootnodes
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

fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	treasury_fund: AccountId,
	cashdrop_fund: AccountId,
	spf_fund: AccountId,
	team_fund: AccountId,
	advisors_and_partners_fund: AccountId,
	labs_faucet: AccountId,
	founder_khalifa_faucet: AccountId,
) -> GenesisConfig {

	let evm_genesis_accounts = evm_genesis();

	let  initial_balance: u128 = 100_000_000 * SETM;
	let  initial_staking: u128 =   10_000_000 * SETM;
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
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_indices: Some(IndicesConfig { indices: vec![] }),
		pallet_balances: Some(BalancesConfig { balances }),
		pallet_session: Some(SessionConfig {
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
		}),
		module_staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), initial_staking, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            force_era: Forcing::NotForcing,
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		}),
        setcloud_market: Some(Default::default()),
        setcloud_swork: Some(SworkConfig {
            init_codes: vec![]
        }),
		pallet_babe: Some(BabeConfig { authorities: vec![] }),
		pallet_grandpa: Some(GrandpaConfig { authorities: vec![] }),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
		pallet_im_online: Default::default(),
		orml_tokens: Some(TokensConfig {
			endowed_accounts: vec![
				// treasury_fund allocation
				(treasury_fund.clone(), SETM, initial_balance),
				(treasury_fund.clone(), SERP, initial_balance),
				(treasury_fund.clone(), DNAR, initial_balance),
				(treasury_fund.clone(), SETR, initial_balance),
				(treasury_fund.clone(), SETUSD, initial_balance),
				(treasury_fund.clone(), RENBTC, initial_balance),
				// cashdrop_fund allocation
				(cashdrop_fund.clone(), SETR, initial_balance),
				(cashdrop_fund.clone(), SETUSD, initial_balance),
				// spf_fund allocation
				(spf_fund.clone(), SETM, initial_balance),
				(spf_fund.clone(), SERP, initial_balance),
				(spf_fund.clone(), DNAR, initial_balance),
				(spf_fund.clone(), SETR, initial_balance),
				(spf_fund.clone(), SETUSD, initial_balance),
				(spf_fund.clone(), RENBTC, initial_balance),
				// team_fund allocation
				(team_fund.clone(), SETM, initial_balance),
				(team_fund.clone(), SERP, initial_balance),
				(team_fund.clone(), DNAR, initial_balance),
				(team_fund.clone(), SETR, initial_balance),
				(team_fund.clone(), SETUSD, initial_balance),
				(team_fund.clone(), RENBTC, initial_balance),
				// advisors_and_partners_fund allocation
				(advisors_and_partners_fund.clone(), SETM, initial_balance),
				(advisors_and_partners_fund.clone(), SERP, initial_balance),
				(advisors_and_partners_fund.clone(), DNAR, initial_balance),
				(advisors_and_partners_fund.clone(), SETR, initial_balance),
				(advisors_and_partners_fund.clone(), SETUSD, initial_balance),
				(advisors_and_partners_fund.clone(), RENBTC, initial_balance),
				// labs_faucet allocation
				(labs_faucet.clone(), SETM, initial_balance),
				(labs_faucet.clone(), SERP, initial_balance),
				(labs_faucet.clone(), DNAR, initial_balance),
				(labs_faucet.clone(), SETR, initial_balance),
				(labs_faucet.clone(), SETUSD, initial_balance),
				(labs_faucet.clone(), RENBTC, initial_balance),
				// founder_khalifa_faucet allocation
				(founder_khalifa_faucet.clone(), SETM, initial_balance),
				(founder_khalifa_faucet.clone(), SERP, initial_balance),
				(founder_khalifa_faucet.clone(), DNAR, initial_balance),
				(founder_khalifa_faucet.clone(), SETR, initial_balance),
				(founder_khalifa_faucet.clone(), SETUSD, initial_balance),
				(founder_khalifa_faucet.clone(), RENBTC, initial_balance),
			],
		}),
		serp_treasury: SerpTreasuryConfig {
			stable_currency_inflation_rate: vec![
				(SETR, 4_000 * dollar(SETR)), 	  // (currency_id, inflation rate of a setcurrency)
				(SETUSD, 8_000 * dollar(SETUSD)), // (currency_id, inflation rate of a setcurrency)
			],
		},
		module_cdp_treasury: CdpTreasuryConfig {
			expected_collateral_auction_size: vec![
				(SETM, dollar(SETM)), 		// (currency_id, max size of a collateral auction)
				(SERP, dollar(SERP)), 		// (currency_id, max size of a collateral auction)
				(DNAR, dollar(DNAR)), 		// (currency_id, max size of a collateral auction)
				(SETR, 5 * dollar(SETR)), 	// (currency_id, max size of a collateral auction)
				(RENBTC, dollar(RENBTC)), 	// (currency_id, max size of a collateral auction)
			],
		},
		module_cdp_engine: CdpEngineConfig {
			collaterals_params: vec![
				(
					SETM,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * dollar(SETUSD),                         // maximum debit value in SETUSD (cap)
				),
				(
					SERP,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * dollar(SETUSD),                         // maximum debit value in SETUSD (cap)
				),
				(
					DNAR,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * dollar(SETUSD),                         // maximum debit value in SETUSD (cap)
				),
				(
					SETR,
					Some(FixedU128::saturating_from_rational(103, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(3, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(106, 100)), // required liquidation ratio
					33_000_000 * dollar(SETUSD),                         // maximum debit value in SETUSD (cap)
				),
				(
					RENBTC,
					Some(FixedU128::saturating_from_rational(115, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(10, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(125, 100)), // required liquidation ratio
					31_300_000 * dollar(SETUSD),                         // maximum debit value in SETUSD (cap)
				),
			],
		},
		module_airdrop: AirDropConfig {
			airdrop_accounts: {
				let setter_airdrop_accounts_json = &include_bytes!("../../../../resources/testnet-airdrop-SETR.json")[..];
				let setter_airdrop_accounts: Vec<(AccountId, Balance)> =
					serde_json::from_slice(setter_airdrop_accounts_json).unwrap();
				let setdollar_airdrop_accounts_json = &include_bytes!("../../../../resources/testnet-airdrop-SETUSD.json")[..];
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
		module_evm: Some(EvmConfig {
			accounts: evm_genesis_accounts,
		}),
		setheum_renvm_bridge: RenVmBridgeConfig {
			ren_vm_public_key: hex!["4b939fc8ade87cb50b78987b1dda927460dc456a"],
		},
		orml_nft: OrmlNFTConfig { tokens: vec![] },
		module_dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		},
		pallet_treasury_instance1: Default::default(),
		pallet_treasury_instance2: Default::default(),
		pallet_sudo: Some(SudoConfig { key: root_key }),
		pallet_collective_Instance1: Default::default(),
		pallet_membership_Instance1: ShuraCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_collective_Instance2: Default::default(),
		pallet_membership_Instance2: FinancialCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_collective_Instance3: Default::default(),
		pallet_membership_Instance3: PublicFundCouncilMembershipConfig {
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
	}
}

fn mainnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	treasury_fund: AccountId,
	cashdrop_fund: AccountId,
	spf_fund: AccountId,
	team_fund: AccountId,
	advisors_and_partners_fund: AccountId,
	labs: AccountId,
	founder_khalifa: AccountId,
) -> GenesisConfig {

	let evm_genesis_accounts = evm_genesis();

	let  setm_foundation_alloc: u128 = 626_600_000 * dollar(SETM);
	let  setm_treasury_alloc: u128 = 313_300_000 * dollar(SETM);
	let  setm_spf_alloc: u128 = 626_600_000 * dollar(SETM);
	let  setm_team_alloc: u128 = 1_315_860_000 * dollar(SETM);
	let  setm_advisors_n_partners_alloc: u128 = 250_640_000 * dollar(SETM);

	let  serp_foundation_alloc: u128 = 51_600_000 * dollar(SERP);
	let  serp_treasury_alloc: u128 = 25_800_000 * dollar(SERP);
	let  serp_spf_alloc: u128 = 51_600_000 * dollar(SERP);
	let  serp_team_alloc: u128 = 108_360_000 * dollar(SERP);
	let  serp_advisors_n_partners_alloc: u128 = 20_640_000 * dollar(SERP);

	let  dnar_foundation_alloc: u128 = 14_000_000 * dollar(DNAR);
	let  dnar_treasury_alloc: u128 = 7_000_000 * dollar(DNAR);
	let  dnar_spf_alloc: u128 = 14_000_000 * dollar(DNAR);
	let  dnar_team_alloc: u128 = 29_400_000 * dollar(DNAR);
	let  dnar_advisors_n_partners_alloc: u128 = 5_600_000 * dollar(DNAR);

	let  setr_foundation_alloc: u128 = 626_600_000 * dollar(SETR);
	let  setr_treasury_alloc: u128 = 313_300_000 * dollar(SETR);
	let  setr_cashdrop_alloc: u128 = 1_315_860_000 * dollar(SETR);
	let  setr_spf_alloc: u128 = 626_600_000 * dollar(SETR);
	let  setr_team_alloc: u128 = 1_315_860_000 * dollar(SETR);
	let  setr_advisors_n_partners_alloc: u128 = 250_640_000 * dollar(SETR);

	let  setusd_foundation_alloc: u128 = 626_600_000 * dollar(SETUSD);
	let  setusd_treasury_alloc: u128 = 313_300_000 * dollar(SETUSD);
	let  setusd_cashdrop_alloc: u128 = 1_315_860_000 * dollar(SETUSD);
	let  setusd_spf_alloc: u128 = 626_600_000 * dollar(SETUSD);
	let  setusd_team_alloc: u128 = 1_315_860_000 * dollar(SETUSD);
	let  setusd_advisors_n_partners_alloc: u128 = 250_640_000 * dollar(SETUSD);

	let initial_staking: u128 = 258_000 * dollar(SETM);
	let existential_deposit = NativeTokenExistentialDeposit::get();
	let mut total_allocated: Balance = Zero::zero();

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), initial_staking + dollar(SETM))) // bit more for fee
		.chain(endowed_accounts.iter().cloned().map(|x| (x.0.clone(), x.1 * dollar(SETM))))
		.chain(
			get_all_module_accounts()
				.iter()
				.map(|x| (x.clone(), existential_deposit)),  // add ED for module accounts
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

	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_indices: Some(IndicesConfig { indices: vec![] }),
		pallet_balances: Some(BalancesConfig { balances }),
		pallet_session: Some(SessionConfig {
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
		}),
		module_staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), initial_staking, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            force_era: Forcing::NotForcing,
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		}),
        setcloud_market: Some(Default::default()),
        setcloud_swork: Some(SworkConfig {
            init_codes: vec![]
        }),
		pallet_babe: Some(BabeConfig { authorities: vec![] }),
		pallet_grandpa: Some(GrandpaConfig { authorities: vec![] }),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
		pallet_im_online: Default::default(),
		orml_tokens: Some(TokensConfig {
			endowed_accounts: vec![
				// Foundation allocation
				(root_key.clone(), SETM, setm_foundation_alloc),
				(root_key.clone(), SERP, serp_foundation_alloc),
				(root_key.clone(), DNAR, dnar_foundation_alloc),
				(root_key.clone(), SETR, setr_foundation_alloc),
				(root_key.clone(), SETUSD, setusd_foundation_alloc),
				// Treasury allocation
				(treasury_fund.clone(), SETM, setm_treasury_alloc),
				(treasury_fund.clone(), SERP, serp_treasury_alloc),
				(treasury_fund.clone(), DNAR, dnar_treasury_alloc),
				(treasury_fund.clone(), SETR, setr_treasury_alloc),
				(treasury_fund.clone(), SETUSD, setusd_treasury_alloc),
				// CashDropFund allocation - Only `SETR` and `SETUSD
				(cashdrop_fund.clone(), SETR, setr_spf_alloc),
				(cashdrop_fund.clone(), SETUSD, setusd_spf_alloc),
				// SPF - PublicFundTreasury allocation
				(spf_fund.clone(), SETM, setm_spf_alloc),
				(spf_fund.clone(), SERP, serp_spf_alloc),
				(spf_fund.clone(), DNAR, dnar_spf_alloc),
				(spf_fund.clone(), SETR, setr_spf_alloc),
				(spf_fund.clone(), SETUSD, setusd_spf_alloc),
				// Team allocation
				(team_fund.clone(), SETM, setm_team_alloc),
				(team_fund.clone(), SERP, serp_team_alloc),
				(team_fund.clone(), DNAR, dnar_team_alloc),
				(team_fund.clone(), SETR, setr_team_alloc),
				(team_fund.clone(), SETUSD, setusd_team_alloc),
				// Advisors and Partners allocation
				(advisors_and_partners_fund.clone(), SETM, setm_advisors_n_partners_alloc),
				(advisors_and_partners_fund.clone(), SERP, serp_advisors_n_partners_alloc),
				(advisors_and_partners_fund.clone(), DNAR, dnar_advisors_n_partners_alloc),
				(advisors_and_partners_fund.clone(), SETR, setr_advisors_n_partners_alloc),
				(advisors_and_partners_fund.clone(), SETUSD, setusd_advisors_n_partners_alloc),
			],
		}),
		serp_treasury: SerpTreasuryConfig {
			stable_currency_inflation_rate: vec![
				(SETR, 4_000 * dollar(SETR)), 	  // (currency_id, inflation rate of a setcurrency)
				(SETUSD, 8_000 * dollar(SETUSD)), // (currency_id, inflation rate of a setcurrency)
			],
		},
		module_cdp_treasury: CdpTreasuryConfig {
			expected_collateral_auction_size: vec![
				(SETM, dollar(SETM)), 		// (currency_id, max size of a collateral auction)
				(SERP, dollar(SERP)), 		// (currency_id, max size of a collateral auction)
				(DNAR, dollar(DNAR)), 		// (currency_id, max size of a collateral auction)
				(SETR, 5 * dollar(SETR)), 	// (currency_id, max size of a collateral auction)
				(RENBTC, dollar(RENBTC)), 	// (currency_id, max size of a collateral auction)
			],
		},
		module_cdp_engine: CdpEngineConfig {
			collaterals_params: vec![
				(
					SETM,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * dollar(SETUSD),                         // maximum debit value in SETUSD (cap)
				),
				(
					SERP,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * dollar(SETUSD),                         // maximum debit value in SETUSD (cap)
				),
				(
					DNAR,
					Some(FixedU128::saturating_from_rational(105, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(5, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(110, 100)), // required liquidation ratio
					25_800_000 * dollar(SETUSD),                         // maximum debit value in SETUSD (cap)
				),
				(
					SETR,
					Some(FixedU128::saturating_from_rational(103, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(3, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(106, 100)), // required liquidation ratio
					33_000_000 * dollar(SETUSD),                         // maximum debit value in SETUSD (cap)
				),
				(
					RENBTC,
					Some(FixedU128::saturating_from_rational(115, 100)), // liquidation ratio
					Some(FixedU128::saturating_from_rational(10, 100)),   // liquidation penalty rate
					Some(FixedU128::saturating_from_rational(125, 100)), // required liquidation ratio
					31_300_000 * dollar(SETUSD),                         // maximum debit value in SETUSD (cap)
				),
			],
		},
		module_airdrop: AirDropConfig {
			airdrop_accounts: {
				let setter_airdrop_accounts_json = &include_bytes!("../../../../resources/mainnet-airdrop-SETR.json")[..];
				let setter_airdrop_accounts: Vec<(AccountId, Balance)> =
					serde_json::from_slice(setter_airdrop_accounts_json).unwrap();
				let setdollar_airdrop_accounts_json = &include_bytes!("../../../../resources/mainnet-airdrop-SETUSD.json")[..];
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
		module_evm: Some(EvmConfig {
			accounts: evm_genesis_accounts,
		}),
		// TODO: Update key for RenVM
		// setheum_renvm_bridge: RenVmBridgeConfig {
		// 	ren_vm_public_key: hex!["4b939fc8ade87cb50b78987b1dda927460dc456a"],
		// },
		orml_nft: OrmlNFTConfig { tokens: vec![] },
		module_dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		},
		pallet_treasury_instance1: Default::default(),
		pallet_treasury_instance2: Default::default(),
		pallet_sudo: Some(SudoConfig { key: root_key }),
		pallet_collective_Instance1: Default::default(),
		pallet_membership_Instance1: ShuraCouncilMembershipConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
				(labs.clone()), 			// Setheum Labs
				(team_fund.clone()), 		// Slixon Technologies
				(founder_khalifa.clone()), 	// Muhammad-Jibril Bin Abdullah-Bashir Al-Sharif. (Khalifa MBA)
			],
			phantom: Default::default(),
		},
		pallet_collective_Instance2: Default::default(),
		pallet_membership_Instance2: FinancialCouncilMembershipConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
				(labs.clone()), 			// Setheum Labs
				(team_fund.clone()), 		// Slixon Technologies
				(founder_khalifa.clone()), 	// Muhammad-Jibril Bin Abdullah-Bashir Al-Sharif. (Khalifa MBA)
			],
			phantom: Default::default(),
		},
		pallet_collective_Instance3: Default::default(),
		pallet_membership_Instance3: PublicFundCouncilMembershipConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
				(labs.clone()), 			// Setheum Labs
				(team_fund.clone()), 		// Slixon Technologies
				(founder_khalifa.clone()), 	// Muhammad-Jibril Bin Abdullah-Bashir Al-Sharif. (Khalifa MBA)
			],
			phantom: Default::default(),
		},
		pallet_collective_Instance4: Default::default(),
		pallet_membership_Instance4: TechnicalCommitteeMembershipConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
				(labs.clone()), 			// Setheum Labs
				(team_fund.clone()), 		// Slixon Technologies
				(founder_khalifa.clone()), 	// Muhammad-Jibril Bin Abdullah-Bashir Al-Sharif. (Khalifa MBA)
			],
			phantom: Default::default(),
		},
		pallet_membership_Instance5: OperatorMembershipSetheumConfig {
			members: vec![
				(root_key.clone()), 		// Setheum Foundation
				(labs.clone()), 			// Setheum Labs
				(team_fund.clone()), 		// Slixon Technologies
				(founder_khalifa.clone()), 	// Muhammad-Jibril Bin Abdullah-Bashir Al-Sharif. (Khalifa MBA)
			],
			phantom: Default::default(),
		},
	}
}


/// Tokens and Currencies Properties
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
	let existential_deposit = NativeTokenExistentialDeposit::get();

	let contracts_json = &include_bytes!("../../predeploy-contracts/resources/bytecodes.json")[..];
	let contracts: Vec<(String, String, String)> = serde_json::from_slice(contracts_json).unwrap();
	let mut accounts = BTreeMap::new();
	for (_, address, code_string) in contracts {
		let account = module_evm::GenesisAccount {
			nonce: 0,
			balance: existential_deposit,
			storage: Default::default(),
			code: Bytes::from_str(&code_string).unwrap().0,
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
