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

//! Mocks for the ECDP module.

#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, derive_impl, ord_parameter_types, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64, Nothing},
	PalletId,
};
use frame_system::{offchain::SendTransactionTypes, EnsureSignedBy};
use module_cdp_engine::CollateralCurrencyIds;
use module_support::{
	EcdpAuctionsManager, ExchangeRate, FractionalRate, Price, PriceProvider, Rate, Ratio, SpecificJointsSwap,
};
use orml_traits::parameter_type_with_key;
use primitives::{
	evm::{convert_decimals_to_evm, EvmAddress},
	Balance, Moment, ReserveIdentifier, TokenSymbol,
};
use sp_core::crypto::AccountId32;
use sp_runtime::{
	testing::TestXt,
	traits::{AccountIdConversion, IdentityLookup, One as OneT},
	BuildStorage, FixedPointNumber,
};
use sp_std::{cell::RefCell, str::FromStr};

mod ecdp {
	pub use super::super::*;
}

pub type AccountId = AccountId32;
pub type BlockNumber = u64;
pub type AuctionId = u32;

pub const ALICE: AccountId = AccountId32::new([1u8; 32]);
pub const BOB: AccountId = AccountId32::new([2u8; 32]);
pub const CAROL: AccountId = AccountId32::new([3u8; 32]);
pub const SEE: CurrencyId = CurrencyId::Token(TokenSymbol::SEE);
pub const USSD: CurrencyId = CurrencyId::Token(TokenSymbol::USSD);
pub const BTC: CurrencyId = CurrencyId::ForeignAsset(255);
pub const EDF: CurrencyId = CurrencyId::Token(TokenSymbol::EDF);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type AccountData = pallet_balances::AccountData<Balance>;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = ReserveIdentifier;
	type DustRemovalWhitelist = Nothing;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = ReserveIdentifier;
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
}
pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, PalletBalances, Amount, BlockNumber>;

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = SEE;
}

impl orml_currencies::Config for Runtime {
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}

parameter_types! {
	pub const EcdpLoansPalletId: PalletId = PalletId(*b"set/ussdloan");
}

impl module_ecdp_loans::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Tokens;
	type RiskManager = EcdpUssdEngineModule;
	type EcdpUssdTreasury = EcdpUssdTreasuryModule;
	type PalletId = EcdpLoansPalletId;
	type OnUpdateLoan = ();
}

pub struct MockPriceSource;
impl PriceProvider<CurrencyId> for MockPriceSource {
	fn get_relative_price(_base: CurrencyId, _quote: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}
}

pub struct MockEcdpAuctionsManager;
impl EcdpAuctionsManager<AccountId> for MockEcdpAuctionsManager {
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AuctionId = AuctionId;

	fn new_collateral_auction(
		_refund_recipient: &AccountId,
		_currency_id: Self::CurrencyId,
		_amount: Self::Balance,
		_target: Self::Balance,
	) -> DispatchResult {
		Ok(())
	}

	fn cancel_auction(_id: Self::AuctionId) -> DispatchResult {
		Ok(())
	}

	fn get_total_target_in_auction() -> Self::Balance {
		Default::default()
	}

	fn get_total_collateral_in_auction(_id: Self::CurrencyId) -> Self::Balance {
		Default::default()
	}
}

thread_local! {
	static IS_SHUTDOWN: RefCell<bool> = RefCell::new(false);
}

pub fn mock_shutdown() {
	IS_SHUTDOWN.with(|v| *v.borrow_mut() = true)
}

pub struct MockEcdpEmergencyShutdown;
impl EcdpEmergencyShutdown for MockEcdpEmergencyShutdown {
	fn is_shutdown() -> bool {
		IS_SHUTDOWN.with(|v| *v.borrow_mut())
	}
}

ord_parameter_types! {
	pub const One: AccountId = AccountId32::new([1u8; 32]);
}

parameter_types! {
	pub const GetUSSDCurrencyId: CurrencyId = USSD;
	pub const EcdpUssdTreasuryPalletId: PalletId = PalletId(*b"set/ussdtrsymod");
	pub TreasuryAccount: AccountId = PalletId(*b"set/ussdtrsyacc").into_account_truncating();
	pub AlternativeSwapPathJointList: Vec<Vec<CurrencyId>> = vec![
		vec![USSD],
	];
}

impl module_ecdp_ussd_treasury::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Currencies;
	type GetUSSDCurrencyId = GetUSSDCurrencyId;
	type EcdpAuctionsManagerHandler = MockEcdpAuctionsManager;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type EdfisSwapHandler = ();
	type Swap = SpecificJointsSwap<(), AlternativeSwapPathJointList>;
	type MaxAuctionsCount = ConstU32<10_000>;
	type PalletId = EcdpUssdTreasuryPalletId;
	type TreasuryAccount = TreasuryAccount;
	type WeightInfo = ();
}

impl pallet_timestamp::Config for Runtime {
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1000>;
	type WeightInfo = ();
}

impl module_evm_accounts::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = PalletBalances;
	type ChainId = ();
	type AddressMapping = module_evm_accounts::EvmAddressMapping<Runtime>;
	type TransferAll = Currencies;
	type WeightInfo = ();
}

parameter_types! {
	pub NetworkContractSource: EvmAddress = EvmAddress::from_str("1000000000000000000000000000000000000001").unwrap();
}

ord_parameter_types! {
	pub const CouncilAccount: AccountId = AccountId::from([1u8; 32]);
	pub const NetworkContractAccount: AccountId = AccountId::from([0u8; 32]);
	pub const StorageDepositPerByte: u128 = convert_decimals_to_evm(10);
}

impl module_evm::Config for Runtime {
	type AddressMapping = module_evm_accounts::EvmAddressMapping<Runtime>;
	type Currency = PalletBalances;
	type TransferAll = ();
	type NewContractExtraBytes = ConstU32<1>;
	type StorageDepositPerByte = StorageDepositPerByte;
	type TxFeePerGas = ConstU128<10>;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type GasToWeight = ();
	type ChargeTransactionPayment = module_support::mocks::MockReservedTransactionPayment<PalletBalances>;
	type NetworkContractOrigin = EnsureSignedBy<NetworkContractAccount, AccountId>;
	type NetworkContractSource = NetworkContractSource;

	type DeveloperDeposit = ConstU128<1000>;
	type PublicationFee = ConstU128<200>;
	type TreasuryAccount = TreasuryAccount;
	type FreePublicationOrigin = EnsureSignedBy<CouncilAccount, AccountId>;

	type Runner = module_evm::runner::stack::Runner<Self>;
	type FindAuthor = ();
	type WeightInfo = ();
}

impl module_evm_bridge::Config for Runtime {
	type EVM = EVM;
}

parameter_type_with_key! {
	pub MinimumCollateralAmount: |_currency_id: CurrencyId| -> Balance {
		10
	};
}

parameter_types! {
	pub DefaultLiquidationRatio: Ratio = Ratio::saturating_from_rational(3, 2);
	pub DefaultDebitExchangeRate: ExchangeRate = ExchangeRate::saturating_from_rational(1, 10);
	pub DefaultLiquidationPenalty: FractionalRate = FractionalRate::try_from(Rate::saturating_from_rational(10, 100)).unwrap();
	pub MaxSwapSlippageCompareToOracle: Ratio = Ratio::saturating_from_rational(50, 100);
	pub MaxLiquidationContractSlippage: Ratio = Ratio::saturating_from_rational(80, 100);
	pub const EcdpUssdEnginePalletId: PalletId = PalletId(*b"set/ussde");
	pub const SettleErc20EvmOrigin: AccountId = AccountId32::new([255u8; 32]);
}

impl module_cdp_engine::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PriceSource = MockPriceSource;
	type DefaultLiquidationRatio = DefaultLiquidationRatio;
	type DefaultDebitExchangeRate = DefaultDebitExchangeRate;
	type DefaultLiquidationPenalty = DefaultLiquidationPenalty;
	type MinimumDebitValue = ConstU128<2>;
	type MinimumCollateralAmount = MinimumCollateralAmount;
	type GetUSSDCurrencyId = GetUSSDCurrencyId;
	type EcdpUssdTreasury = EcdpUssdTreasuryModule;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type MaxSwapSlippageCompareToOracle = MaxSwapSlippageCompareToOracle;
	type UnsignedPriority = ConstU64<1048576>; // 1 << 20
	type EcdpEmergencyShutdown = MockEcdpEmergencyShutdown;
	type UnixTime = Timestamp;
	type Currency = Currencies;
	type EdfisSwapHandler = ();
	type LiquidationContractsUpdateOrigin = EnsureSignedBy<One, AccountId>;
	type MaxLiquidationContractSlippage = MaxLiquidationContractSlippage;
	type MaxLiquidationContracts = ConstU32<10>;
	type LiquidationEvmBridge = ();
	type PalletId = EcdpUssdEnginePalletId;
	type EvmAddressMapping = module_evm_accounts::EvmAddressMapping<Runtime>;
	type Swap = SpecificJointsSwap<(), AlternativeSwapPathJointList>;
	type EVMBridge = module_evm_bridge::EVMBridge<Runtime>;
	type SettleErc20EvmOrigin = SettleErc20EvmOrigin;
	type WeightInfo = ();
}

type Block = frame_system::mocking::MockBlock<Runtime>;

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = PalletBalances;
	type DepositPerAuthorization = ConstU128<100>;
	type CollateralCurrencyIds = CollateralCurrencyIds<Runtime>;
	type WeightInfo = ();
}

construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		EcdpModule: ecdp,
		Tokens: orml_tokens,
		PalletBalances: pallet_balances,
		Currencies: orml_currencies,
		EcdpLoansModule: module_ecdp_loans,
		EcdpUssdTreasuryModule: module_ecdp_ussd_treasury,
		EcdpUssdEngineModule: module_cdp_engine,
		Timestamp: pallet_timestamp,
		EvmAccounts: module_evm_accounts,
		EVM: module_evm,
		EVMBridge: module_evm_bridge,
	}
);

/// An extrinsic type used for tests.
pub type Extrinsic = TestXt<RuntimeCall, ()>;

impl<LocalCall> SendTransactionTypes<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = Extrinsic;
}

pub struct ExtBuilder {
	endowed_native: Vec<(AccountId, Balance)>,
	balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_native: vec![(ALICE, 1000)],
			balances: vec![
				(ALICE, BTC, 1000),
				(BOB, BTC, 1000),
				(ALICE, EDF, 1000),
				(BOB, EDF, 1000),
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.endowed_native,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
