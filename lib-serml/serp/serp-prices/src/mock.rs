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

//! Mocks for the prices module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use orml_traits::{parameter_type_with_key, DataFeeder};
use primitives::{Amount, TokenSymbol};
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{IdentityLookup, One as OneT, Zero},
	DispatchError, FixedPointNumber,
};
use support::{mocks::MockCurrencyIdMapping, Ratio};

pub type AccountId = u128;
pub type BlockNumber = u64;

mod serp_prices {
	pub use super::super::*;
}

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const DRAM: CurrencyId = CurrencyId::Token(TokenSymbol::DRAM);
pub const SETT: CurrencyId = CurrencyId::Token(TokenSymbol::SETT);
pub const AUDJ: CurrencyId = CurrencyId::Token(TokenSymbol::AUDJ);
pub const CADJ: CurrencyId = CurrencyId::Token(TokenSymbol::CADJ);
pub const CHFJ: CurrencyId = CurrencyId::Token(TokenSymbol::CHFJ);
pub const EURJ: CurrencyId = CurrencyId::Token(TokenSymbol::EURJ);
pub const GBPJ: CurrencyId = CurrencyId::Token(TokenSymbol::GBPJ);
pub const JPYJ: CurrencyId = CurrencyId::Token(TokenSymbol::JPYJ);
pub const SARJ: CurrencyId = CurrencyId::Token(TokenSymbol::SARJ);
pub const SEKJ: CurrencyId = CurrencyId::Token(TokenSymbol::SEKJ);
pub const SGDJ: CurrencyId = CurrencyId::Token(TokenSymbol::SGDJ);
pub const USDJ: CurrencyId = CurrencyId::Token(TokenSymbol::USDJ);
pub const BTC: CurrencyId = CurrencyId::Token(TokenSymbol::RENBTC);


// LP tokens constants - CurrencyId/TokenSymbol : Dex Shares
pub const LP_BTC_USDJ: CurrencyId =
CurrencyId::DexShare(DexShare::Token(TokenSymbol::RENBTC), DexShare::Token(TokenSymbol::USDJ));
pub const LP_USDJ_DNAR: CurrencyId =
CurrencyId::DexShare(DexShare::Token(TokenSymbol::USDJ), DexShare::Token(TokenSymbol::DNAR));
pub const LP_USDJ_DRAM: CurrencyId =
CurrencyId::DexShare(DexShare::Token(TokenSymbol::USDJ), DexShare::Token(TokenSymbol::DRAM));

// Currencies constants - FiatCurrencyIds (CurrencyId/TokenSymbol)
pub const AUD: CurrencyId = CurrencyId::Token(TokenSymbol::AUD);
pub const CAD: CurrencyId = CurrencyId::Token(TokenSymbol::CAD);
pub const CHF: CurrencyId = CurrencyId::Token(TokenSymbol::CHF);
pub const EUR: CurrencyId = CurrencyId::Token(TokenSymbol::EUR);
pub const GBP: CurrencyId = CurrencyId::Token(TokenSymbol::GBP);
pub const JPY: CurrencyId = CurrencyId::Token(TokenSymbol::JPY);
pub const SAR: CurrencyId = CurrencyId::Token(TokenSymbol::SAR);
pub const SEK: CurrencyId = CurrencyId::Token(TokenSymbol::SEK);
pub const SGD: CurrencyId = CurrencyId::Token(TokenSymbol::SGD);
pub const USD: CurrencyId = CurrencyId::Token(TokenSymbol::USD);
pub const KWD: CurrencyId = CurrencyId::Token(TokenSymbol::KWD);
pub const JOD: CurrencyId = CurrencyId::Token(TokenSymbol::JOD);
pub const BHD: CurrencyId = CurrencyId::Token(TokenSymbol::BHD);
pub const KYD: CurrencyId = CurrencyId::Token(TokenSymbol::KYD);
pub const OMR: CurrencyId = CurrencyId::Token(TokenSymbol::OMR);
pub const GIP: CurrencyId = CurrencyId::Token(TokenSymbol::GIP);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

pub struct MockDataProvider;
impl DataProvider<CurrencyId, Price> for MockDataProvider {
	fn get(currency_id: &CurrencyId) -> Option<Price> {
		match *currency_id {
			USDJ => Some(Price::saturating_from_rational(99, 100)),
			BTC => Some(Price::saturating_from_integer(50000)),
			DNAR => Some(Price::saturating_from_integer(100)),
			DRAM => Some(Price::zero()),
			_ => None,
		}
	}
}

impl DataFeeder<CurrencyId, Price, AccountId> for MockDataProvider {
	fn feed_value(_: AccountId, _: CurrencyId, _: Price) -> sp_runtime::DispatchResult {
		Ok(())
	}
}

pub struct MockDEX;
impl DEXManager<AccountId, CurrencyId, Balance> for MockDEX {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance) {
		match (currency_id_a, currency_id_b) {
			(USDJ, DNAR) => (10000, 200),
			_ => (0, 0),
		}
	}

	fn get_liquidity_token_address(_currency_id_a: CurrencyId, _currency_id_b: CurrencyId) -> Option<H160> {
		unimplemented!()
	}

	fn get_swap_target_amount(
		_path: &[CurrencyId],
		_supply_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> Option<Balance> {
		unimplemented!()
	}

	fn get_swap_supply_amount(
		_path: &[CurrencyId],
		_target_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> Option<Balance> {
		unimplemented!()
	}

	fn swap_with_exact_supply(
		_who: &AccountId,
		_path: &[CurrencyId],
		_supply_amount: Balance,
		_min_target_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn swap_with_exact_target(
		_who: &AccountId,
		_path: &[CurrencyId],
		_target_amount: Balance,
		_max_supply_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn add_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_max_amount_a: Balance,
		_max_amount_b: Balance,
		_min_share_increment: Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn remove_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_remove_share: Balance,
		_min_withdrawn_a: Balance,
		_min_withdrawn_b: Balance,
	) -> DispatchResult {
		unimplemented!()
	}
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = ();
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub const SetterCurrencyId: CurrencyId = SETT; // Setter currency ticker is SETT.
	pub const GetSettUSDCurrencyId: CurrencyId = USDJ; // SettUSD currency ticker is USDJ.
	pub const GetFiatAUDCurrencyId: CurrencyId = AUD; // The AUD Fiat currency denomination.
	pub const GetFiatCADCurrencyId: CurrencyId = CAD; // The CAD Fiat currency denomination.
	pub const GetFiatCHFCurrencyId: CurrencyId = CHF; // The CHF Fiat currency denomination.
	pub const GetFiatEURCurrencyId: CurrencyId = EUR; // The EUR Fiat currency denomination.
	pub const GetFiatGBPCurrencyId: CurrencyId = GBP; // The GBP Fiat currency denomination.
	pub const GetFiatJPYCurrencyId: CurrencyId = JPY; // The JPY Fiat currency denomination.
	pub const GetFiatSARCurrencyId: CurrencyId = SAR; // The SAR Fiat currency denomination.
	pub const GetFiatSEKCurrencyId: CurrencyId = SEK; // The SEK Fiat currency denomination.
	pub const GetFiatSGDCurrencyId: CurrencyId = SGD; // The SGD Fiat currency denomination.
	pub const GetFiatUSDCurrencyId: CurrencyId = USD; // The USD Fiat currency denomination.
	pub FiatUsdFixedPrice: Price = Price::one(); // Fixed 1 USD Fiat denomination for pricing.

	pub const GetSetterPegOneCurrencyId: CurrencyId = GBP; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegTwoCurrencyId: CurrencyId = EUR; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegThreeCurrencyId: CurrencyId = KWD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegFourCurrencyId: CurrencyId = JOD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegFiveCurrencyId: CurrencyId = BHD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegSixCurrencyId: CurrencyId = KYD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegSevenCurrencyId: CurrencyId = OMR; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegEightCurrencyId: CurrencyId = CHF; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegNineCurrencyId: CurrencyId = GIP; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegTenCurrencyId: CurrencyId = USD; // Fiat pegs of the Setter (SETT).
	
	
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETT, AUDJ, CADJ, BTC, EURJ, GBPJ,
		JPYJ, SARJ, SEKJ, SGDJ, USDJ,
	];
	pub FiatCurrencyIds: Vec<CurrencyId> = vec![
		AUD, CAD, CHF, EUR, GBP, JPY, SAR, SEK,
		SGD, USD, JOD, BHD, KYD, OMR, GIP
	];
}

impl Config for Runtime {
	type Event = Event;
	type Source = MockDataProvider;
	type SetterCurrencyId = SetterCurrencyId;
	type GetSettUSDCurrencyId = GetSettUSDCurrencyId;
	type GetFiatAUDCurrencyId = GetFiatAUDCurrencyId;
	type GetFiatCADCurrencyId = GetFiatCADCurrencyId;
	type GetFiatCHFCurrencyId = GetFiatCHFCurrencyId;
	type GetFiatEURCurrencyId = GetFiatEURCurrencyId;
	type GetFiatGBPCurrencyId = GetFiatGBPCurrencyId;
	type GetFiatJPYCurrencyId = GetFiatJPYCurrencyId;
	type GetFiatSARCurrencyId = GetFiatSARCurrencyId;
	type GetFiatSEKCurrencyId = GetFiatSEKCurrencyId;
	type GetFiatSGDCurrencyId = GetFiatSGDCurrencyId;
	type GetFiatUSDCurrencyId = GetFiatUSDCurrencyId;
	type FiatUsdFixedPrice = FiatUsdFixedPrice;
	type GetSetterPegOneCurrencyId = GetSetterPegOneCurrencyId;
	type GetSetterPegTwoCurrencyId = GetSetterPegTwoCurrencyId;
	type GetSetterPegThreeCurrencyId = GetSetterPegThreeCurrencyId;
	type GetSetterPegFourCurrencyId = GetSetterPegFourCurrencyId;
	type GetSetterPegFiveCurrencyId = GetSetterPegFiveCurrencyId;
	type GetSetterPegSixCurrencyId = GetSetterPegSixCurrencyId;
	type GetSetterPegSevenCurrencyId = GetSetterPegSevenCurrencyId;
	type GetSetterPegEightCurrencyId = GetSetterPegEightCurrencyId;
	type GetSetterPegNineCurrencyId = GetSetterPegNineCurrencyId;
	type GetSetterPegTenCurrencyId = GetSetterPegTenCurrencyId;
	type LockOrigin = EnsureSignedBy<One, AccountId>;
	type DEX = MockDEX;
	type Currency = Tokens;
	type CurrencyIdMapping = MockCurrencyIdMapping;
	type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		SerpPrices: serp_prices::{Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Event<T>},
	}
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		t.into()
	}
}
