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

//! Mocks for the prices module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use orml_traits::{parameter_type_with_key, DataFeeder};
use primitives::{Amount, TokenSymbol};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{IdentityLookup, One as OneT, Zero},
	DispatchError, FixedPointNumber,
};
use support::{ExchangeRate, Ratio};

pub type AccountId = u128;
pub type BlockNumber = u64;

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const SDEX: CurrencyId = CurrencyId::Token(TokenSymbol::SDEX); //  SettinDex
pub const SETT: CurrencyId = CurrencyId::Token(TokenSymbol::SETT); // Setter   -  The Defacto stablecoin & settmint reserve asset
pub const USDJ: CurrencyId = CurrencyId::Token(TokenSymbol::USDJ); // Setheum USD (US Dollar stablecoin)
pub const GBPJ: CurrencyId = CurrencyId::Token(TokenSymbol::GBPJ); // Setheum GBP (Pound Sterling stablecoin)
pub const EURJ: CurrencyId = CurrencyId::Token(TokenSymbol::EURJ); // Setheum EUR (Euro stablecoin)
pub const KWDJ: CurrencyId = CurrencyId::Token(TokenSymbol::KWDJ); // Setheum KWD (Kuwaiti Dinar stablecoin)
pub const JODJ: CurrencyId = CurrencyId::Token(TokenSymbol::JODJ); // Setheum JOD (Jordanian Dinar stablecoin)
pub const BHDJ: CurrencyId = CurrencyId::Token(TokenSymbol::BHDJ); // Setheum BHD (Bahraini Dirham stablecoin)
pub const KYDJ: CurrencyId = CurrencyId::Token(TokenSymbol::KYDJ); // Setheum KYD (Cayman Islands Dollar stablecoin)
pub const OMRJ: CurrencyId = CurrencyId::Token(TokenSymbol::OMRJ); // Setheum OMR (Omani Riyal stablecoin)
pub const CHFJ: CurrencyId = CurrencyId::Token(TokenSymbol::CHFJ); // Setheum CHF (Swiss Franc stablecoin)
pub const GIPJ: CurrencyId = CurrencyId::Token(TokenSymbol::GIPJ); // Setheum GIP (Gibraltar Pound stablecoin)

// LP tokens constants - CurrencyId/TokenSymbol : Dex Shares
pub const LP_CHFJ_USDJ: CurrencyId = CurrencyId::DexShare(TokenSymbol::CHFJ, TokenSymbol::USDJ);
pub const LP_USDJ_DNAR: CurrencyId = CurrencyId::DexShare(TokenSymbol::USDJ, TokenSymbol::DNAR);

// Currencies constants - FiatCurrencyIds
pub const USD: FiatCurrencyId = USD; // 1.  US Dollar 			  (Fiat - only for price feed)
pub const GBP: FiatCurrencyId = GBP; // 2.  Pound Sterling 		  (Fiat - only for price feed)
pub const EUR: FiatCurrencyId = EUR; // 3.  Euro 				  (Fiat - only for price feed)
pub const KWD: FiatCurrencyId = KWD; // 4.  Kuwaiti Dinar 		  (Fiat - only for price feed)
pub const JOD: FiatCurrencyId = JOD; // 5.  Jordanian Dinar 	  (Fiat - only for price feed)
pub const BHD: FiatCurrencyId = BHD; // 6.  Bahraini Dirham 	  (Fiat - only for price feed)
pub const KYD: FiatCurrencyId = KYD; // 7.  Cayman Islands Dollar (Fiat - only for price feed)
pub const OMR: FiatCurrencyId = OMR; // 8.  Omani Riyal 		  (Fiat - only for price feed)
pub const CHF: FiatCurrencyId = CHF; // 9.  Swiss Franc 		  (Fiat - only for price feed)
pub const GIP: FiatCurrencyId = GIP; // 10. Gibraltar Pound 	  (Fiat - only for price feed)

mod setheum_prices {
	pub use super::super::*;
}

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
}

pub struct MockDataProvider;
impl DataProvider<CurrencyId, Price> for MockDataProvider {
	fn get(currency_id: &CurrencyId) -> Option<Price> {
		match *currency_id {
			USDJ => Some(Price::saturating_from_rational(99, 100)),
			CHFJ => Some(Price::saturating_from_integer(50000)),
			DNAR => Some(Price::saturating_from_integer(100)),
			DNAR => Some(Price::zero()),
			_ => None,
		}
	}
}

impl DataFeeder<CurrencyId, Price, AccountId> for MockDataProvider {
	fn feed_value(_: AccountId, _: CurrencyId, _: Price) -> sp_runtime::DispatchResult {
		Ok(())
	}
}

pub struct MockDex;
impl DexManager<AccountId, CurrencyId, Balance> for MockDex {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance) {
		match (currency_id_a, currency_id_b) {
			(USDJ, DNAR) => (10000, 200),
			_ => (0, 0),
		}
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

	fn add_liquidity(_: &AccountId, _: CurrencyId, _: CurrencyId, _: Balance, _: Balance, _: bool) -> DispatchResult {
		unimplemented!()
	}

	fn remove_liquidity(_: &AccountId, _: CurrencyId, _: CurrencyId, _: Balance, _: bool) -> DispatchResult {
		unimplemented!()
	}
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};

	pub PegCurrencyIds: |_currency_id: CurrencyId| -> Balance {
		match currency_id {
			&USDJ => &USD,
			&GBPJ => &GBP,
			&EURJ => &EUR,
			&KWDJ => &KWD,
			&JODJ => &JOD,
			&BHDJ => &BHD,
			&KYDJ => &KYD,
			&OMRJ => &OMR,
			&CHFJ => &CHF,
			&GIPJ => &GIP,
			_ => 0,
		}
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
	pub const GetSetterCurrencyId: CurrencyId = SETT; // Setter currency ticker is SETT
	pub const GetSettUSDCurrencyId: CurrencyId = USDJ; // SettUSD currency ticker is USDJ
	pub const GetSettGBPCurrencyId: CurrencyId = GBPJ; // SettGBP currency ticker is GBPJ
	pub const GetSettEURCurrencyId: CurrencyId = EURJ; // SettEUR currency ticker is EURJ
	pub const GetSettKWDCurrencyId: CurrencyId = KWDJ; // SettKWD currency ticker is KWDJ
	pub const GetSettJODCurrencyId: CurrencyId = JODJ; // SettJOD currency ticker is JODJ
	pub const GetSettBHDCurrencyId: CurrencyId = BHDJ; // SettBHD currency ticker is BHDJ
	pub const GetSettKYDCurrencyId: CurrencyId = KYDJ; // SettKYD currency ticker is KYDJ
	pub const GetSettOMRCurrencyId: CurrencyId = OMRJ; // SettOMR currency ticker is OMRJ
	pub const GetSettCHFCurrencyId: CurrencyId = CHFJ; // SettCHF currency ticker is CHFJ
	pub const GetSettGIPCurrencyId: CurrencyId = GIPJ; // SettGIP currency ticker is GIPJ
	pub SettUSDFixedPrice: Price = Price::one(); // All prices are in USD. USDJ is pegged 1:1 to USD
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETT, // Setter   -  The Defacto stablecoin & settmint reserve asset
		USDJ, // Setheum USD (US Dollar stablecoin)
		GBPJ, // Setheum GBP (Pound Sterling stablecoin)
		EURJ, // Setheum EUR (Euro stablecoin)
		KWDJ, // Setheum KWD (Kuwaiti Dinar stablecoin)
		JODJ, // Setheum JOD (Jordanian Dinar stablecoin)
		BHDJ, // Setheum BHD (Bahraini Dirham stablecoin)
		KYDJ, // Setheum KYD (Cayman Islands Dollar stablecoin)
		OMRJ, // Setheum OMR (Omani Riyal stablecoin)
		CHFJ, // Setheum CHF (Swiss Franc stablecoin)
		GIPJ, // Setheum GIP (Gibraltar Pound stablecoin)
	];
	pub FiatCurrencyIds: Vec<CurrencyId> = vec![
		USD, // US Dollar 			  (Fiat - only for price feed)
		GBP, // Pound Sterling 		  (Fiat - only for price feed)
		EUR, // Euro 				  (Fiat - only for price feed)
		KWD, // Kuwaiti Dinar 		  (Fiat - only for price feed)
		JOD, // Jordanian Dinar 	  (Fiat - only for price feed)
		BHD, // Bahraini Dirham 	  (Fiat - only for price feed)
		KYD, // Cayman Islands Dollar (Fiat - only for price feed)
		OMR, // Omani Riyal 		  (Fiat - only for price feed)
		CHF, // Swiss Franc 		  (Fiat - only for price feed)
		GIP, // Gibraltar Pound 	  (Fiat - only for price feed)
		];
}

impl Config for Runtime {
	type Event = Event;
	type Source = MockDataProvider;
	type GetSetterCurrencyId = GetSetterCurrencyId;
	type GetSettUSDCurrencyId = GetSettUSDCurrencyId;
	type GetSettGBPCurrencyId = GetSettGBPCurrencyId;
	type GetSettEURCurrencyId = GetSettEURCurrencyId;
	type GetSettKWDCurrencyId = GetSettKWDCurrencyId;
	type GetSettJODCurrencyId = GetSettJODCurrencyId;
	type GetSettBHDCurrencyId = GetSettBHDCurrencyId;
	type GetSettKYDCurrencyId = GetSettKYDCurrencyId;
	type GetSettOMRCurrencyId = GetSettOMRCurrencyId;
	type GetSettCHFCurrencyId = GetSettCHFCurrencyId;
	type GetSettGIPCurrencyId = GetSettGIPCurrencyId;
	type SettUSDFixedPrice = SettUSDFixedPrice;
	type StableCurrencyIds = StableCurrencyIds;
	type FiatCurrencyIds = FiatCurrencyIds;
	type LockOrigin = EnsureSignedBy<One, AccountId>;
	type Dex = MockDex;
	type Currency = Tokens;
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
		PricesModule: setheum_prices::{Pallet, Storage, Call, Event<T>},
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
