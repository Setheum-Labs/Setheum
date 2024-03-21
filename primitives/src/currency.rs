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

#![allow(clippy::from_over_into)]

use crate::{evm::EvmAddress, *};
use bstringify::bstringify;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use serde::{Deserialize, Serialize};

macro_rules! create_currency_id {
    ($(#[$meta:meta])*
	$vis:vis enum TokenSymbol {
        $($(#[$vmeta:meta])* $symbol:ident($name:expr, $deci:literal) = $val:literal,)*
    }) => {
		$(#[$meta])*
		$vis enum TokenSymbol {
			$($(#[$vmeta])* $symbol = $val,)*
		}

		impl TryFrom<u8> for TokenSymbol {
			type Error = ();

			fn try_from(v: u8) -> Result<Self, Self::Error> {
				match v {
					$($val => Ok(TokenSymbol::$symbol),)*
					_ => Err(()),
				}
			}
		}

		impl Into<u8> for TokenSymbol {
			fn into(self) -> u8 {
				match self {
					$(TokenSymbol::$symbol => ($val),)*
				}
			}
		}

		impl TryFrom<Vec<u8>> for CurrencyId {
			type Error = ();
			fn try_from(v: Vec<u8>) -> Result<CurrencyId, ()> {
				match v.as_slice() {
					$(bstringify!($symbol) => Ok(CurrencyId::Token(TokenSymbol::$symbol)),)*
					_ => Err(()),
				}
			}
		}

		impl TokenInfo for CurrencyId {
			fn currency_id(&self) -> Option<u8> {
				match self {
					$(CurrencyId::Token(TokenSymbol::$symbol) => Some($val),)*
					_ => None,
				}
			}
			fn name(&self) -> Option<&str> {
				match self {
					$(CurrencyId::Token(TokenSymbol::$symbol) => Some($name),)*
					_ => None,
				}
			}
			fn symbol(&self) -> Option<&str> {
				match self {
					$(CurrencyId::Token(TokenSymbol::$symbol) => Some(stringify!($symbol)),)*
					_ => None,
				}
			}
			fn decimals(&self) -> Option<u8> {
				match self {
					$(CurrencyId::Token(TokenSymbol::$symbol) => Some($deci),)*
					_ => None,
				}
			}
		}

		$(pub const $symbol: CurrencyId = CurrencyId::Token(TokenSymbol::$symbol);)*

		impl TokenSymbol {
			pub fn get_info() -> Vec<(&'static str, u32)> {
				vec![
					$((stringify!($symbol), $deci),)*
				]
			}
		}

		#[test]
		#[ignore]
		fn generate_token_resources() {
			use crate::TokenSymbol::*;

			#[allow(non_snake_case)]
			#[derive(Serialize, Deserialize)]
			struct Token {
				symbol: String,
				address: EvmAddress,
			}

			// Setheum tokens
			let mut tokens = vec![];
			$(
				if $val < 128 {
					tokens.push(Token {
						symbol: stringify!($symbol).to_string(),
						address: EvmAddress::try_from(CurrencyId::Token(TokenSymbol::$symbol)).unwrap(),
					});
				}
			)*

			let mut lp_tokens = vec![
				// SETR PAIRED POOLS
				Token {
					symbol: "LP_SEE_SETR".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(SETR), CurrencyId::Token(SEE)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_EDF_SETR".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(SETR), CurrencyId::Token(EDF)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_LSEE_SETR".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(SETR), CurrencyId::Token(LSEE)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_LEDF_SETR".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(SETR), CurrencyId::Token(LEDF)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_USSD_SETR".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(SETR), CurrencyId::Token(USSD)).unwrap().dex_share_currency_id()).unwrap(),
				},
			];
			tokens.append(&mut lp_tokens);

			let mut fa_tokens = vec![
				Token {
					symbol: "FA_WBTC".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(5)).unwrap(),
				},
				Token {
					symbol: "FA_WETH".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(6)).unwrap(),
				},
			];
			tokens.append(&mut fa_tokens);

			frame_support::assert_ok!(std::fs::write("../../blockchain/predeploy-contracts/resources/tokens.json", serde_json::to_string_pretty(&tokens).unwrap()));
		}
    }
}

create_currency_id! {
	// Represent a Token symbol with 8 bit
	//
	// 0 - 19: Setheum native tokens
	// 20 - 255: Reserved for future usage
	#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo, MaxEncodedLen, Serialize, Deserialize)]
	#[repr(u8)]
	pub enum TokenSymbol {
		// 0 - 100: Reserved for Setheum Native Assets
			// Primary Protocol Tokens
		SEE("Setheum", 12) = 0,
		EDF("Ethical DeFi", 12) = 1,
			// Liquid Staking Tokens
		LSEE("Liquid SEE", 12) = 2,
		LEDF("Liquid EDF", 12) = 3,
			// ECDP Stablecoin Tokens
		SETR("Setter", 12) = 4,
		USSD("Slick USD", 12) = 5,

		// 101-255: Reserved for Fiat Currencies
		AED("UAE Dirham", 2) = 101,
		AMD("Armenian Dram", 2) = 102,
		AOA("Angolan Kwanza", 2) = 103,
		ARS("Argentine Peso", 2) = 104,
		AUD("Australian Dollar", 2) = 105,
		AZN("Azerbaijani Manat", 2) = 106,
		BHD("Bahraini Dinar", 3) = 107,
		BIF("Burundian Franc", 2) = 108,
		BND("Brunei Dollar", 2) = 109,
		BRL("Brazilian Real", 2) = 110,
		BSD("Bahamian Dollar", 2) = 111,
		BWP("Botswana Pula", 2) = 112,
		BYN("Belarusian Ruble", 2) = 113,
		CAD("Canadian Dollar", 2) = 114,
		CHF("Swiss Franc", 2) = 115,
		CLP("Chilean Peso", 2) = 116,
		CNY("Chinese Renminbi", 2) = 117,
		COM("Comorian Franc", 2) = 118,
		COP("Colombian Peso", 2) = 119,
		CRC("Costa Rican Colón", 2) = 120,
		CUP("Cuban Peso", 2) = 121,
		CVE("Cape Verdean Escudo", 2) = 122,
		CZK("Czech Koruna", 2) = 123,
		DJF("Djiboutian Franc", 2) = 124,
		DKK("Danish Krone", 2) = 125,
		DOP("Dominican Peso", 2) = 126,
		DZD("Algerian Dinar", 2) = 127,
		EGP("Egyptian Pound", 2) = 128,
		ERN("Eritrean Nakfa", 2) = 129,
		ETB("Ethiopian Birr", 2) = 130,
		EUR("Euro", 2) = 131,
		GBP("British Pound", 2) = 132,
		GEL("Georgian Lari", 2) = 133,
		GHS("Ghanaian Cedi", 2) = 134,
		GMD("Gambian Dalasi", 2) = 135,
		GNF("Guinean Franc", 2) = 136,
		HKD("Hong Kong Dollar", 2) = 137,
		HUF("Hungarian Forint", 2) = 138,
		IDR("Indonesian Rupiah", 2) = 139,
		INR("Indian Rupee", 2) = 140,
		ISK("Icelandic Krona", 2) = 141,
		JOD("Jordanian Dinar", 2) = 142,
		JPY("Japanese Yen", 2) = 143,
		KES("Kenyan Shilling", 2) = 144,
		KHR("Cambodian Riel", 2) = 145,
		KMF("Comorian Franc", 2) = 146,
		KRW("South Korean Won", 2) = 147,
		KWD("Kuwaiti Dinar", 2) = 148,
		KZT("Kazakhstani Tenge", 2) = 149,
		LBP("Lebanese Pound", 2) = 150,
		LKR("Sri Lankan Rupee", 2) = 151,
		LSL("Lesotho Loti", 2) = 152,
		LRD("Liberian Dollar", 2) = 153,
		MAD("Moroccan Dirham", 2) = 154,
		MDL("Moldovan Leu", 2) = 155,
		MGA("Malagasy Ariary", 2) = 156,
		MNT("Mongolian Tugrik", 2) = 157,
		MRU("Mauritanian Ouguiya", 2) = 158,
		MUR("Mauritian Rupee", 2) = 159,
		MWK("Malawian Kwacha", 2) = 160,
		MXN("Mexican Peso", 2) = 161,
		MYR("Malaysian Ringgit", 2) = 162,
		MZN("Mozambican Metical", 2) = 163,
		NAD("Namibian Dollar", 2) = 164,
		NGN("Nigerian Naira", 2) = 165,
		NOK("Norwegian Krone", 2) = 166,
		NPR("Nepalese Rupee", 2) = 167,
		NZD("New Zealand Dollar", 2) = 168,
		OMR("Omani Rial", 2) = 169,
		PEN("Peruvian Sol", 2) = 170,
		PHP("Philippine Peso", 2) = 171,
		PKR("Pakistani Rupee", 2) = 172,
		QAR("Qatari Riyal", 2) = 173,
		RON("Romanian Leu", 2) = 174,
		RSD("Serbian Dinar", 2) = 175,
		RUB("Russian Ruble", 2) = 176,
		RWF("Rwandan Franc", 2) = 177,
		SAR("Saudi Riyal", 2) = 178,
		SCR("Seychellois Rupee", 2) = 179,
		SEK("Swedish Krona", 2) = 180,
		SGD("Singapore Dollar", 2) = 181,
		SHP("Saint Helena Pound", 2) = 182,
		SLE("Sierra Leonean Leone", 2) = 183,
		SZL("Swazi Lilangeni", 2) = 184,
		THB("Thai Baht", 2) = 185,
		TJS("Tajikistani Somoni", 2) = 186,
		TND("Tunisian Dinar", 2) = 187,
		TTD("Trinidadian Dollar", 2) = 188,
		TWD("New Taiwan Dollar", 2) = 189,
		TZS("Tanzanian Shilling", 2) = 190,
		TRY("Turkish Lira", 2) = 191,
		UAH("Ukrainian Hryvnia", 2) = 192,
		UGX("Ugandan Shilling", 2) = 193,
		USD("United States Dollar", 2) = 194,
		UZS("Uzbekistani Som", 2) = 195,
		VES("Venezuelan Bolivar", 2) = 196,
		VND("Vietnamese Dong", 2) = 197,
		XAF("Central African CFA Franc", 2) = 198,
		XOF("West African CFA Franc", 2) = 199,
		ZAR("South African Rand", 2) = 200,
		ZMW("Zambian Kwacha", 2) = 201,
		ZWL("Zimbabwean Dollar", 2) = 202,
	}
}

pub trait TokenInfo {
	fn currency_id(&self) -> Option<u8>;
	fn name(&self) -> Option<&str>;
	fn symbol(&self) -> Option<&str>;
	fn decimals(&self) -> Option<u8>;
}

pub type FiatCurrencyId = u8;
pub type ForeignAssetId = u16;
pub type Erc20Id = u32;

#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	TypeInfo,
	MaxEncodedLen,
	Serialize,
	Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum DexShare {
	Token(TokenSymbol),
	Erc20(EvmAddress),
	ForeignAsset(ForeignAssetId),
}

#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	TypeInfo,
	MaxEncodedLen,
	Serialize,
	Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum CurrencyId {
	Token(TokenSymbol),
	DexShare(DexShare, DexShare),
	Erc20(EvmAddress),
	ForeignAsset(ForeignAssetId),
	FiatCurrency(FiatCurrencyId),
}

impl CurrencyId {
	pub fn is_token_currency_id(&self) -> bool {
		matches!(self, CurrencyId::Token(_))
	}

	pub fn is_dex_share_currency_id(&self) -> bool {
		matches!(self, CurrencyId::DexShare(_, _))
	}

	pub fn is_erc20_currency_id(&self) -> bool {
		matches!(self, CurrencyId::Erc20(_))
	}

	pub fn is_foreign_asset_currency_id(&self) -> bool {
		matches!(self, CurrencyId::ForeignAsset(_))
	}

	pub fn is_fiat_asset_currency_id(&self) -> bool {
		matches!(self, CurrencyId::FiatCurrency(_))
	}

	pub fn is_trading_pair_currency_id(&self) -> bool {
		matches!(
			self,
			CurrencyId::Token(_)
				| CurrencyId::Erc20(_)
				| CurrencyId::ForeignAsset(_)
		)
	}

	pub fn split_dex_share_currency_id(&self) -> Option<(Self, Self)> {
		match self {
			CurrencyId::DexShare(dex_share_0, dex_share_1) => {
				let currency_id_0: CurrencyId = (*dex_share_0).into();
				let currency_id_1: CurrencyId = (*dex_share_1).into();
				Some((currency_id_0, currency_id_1))
			}
			_ => None,
		}
	}

	pub fn join_dex_share_currency_id(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
		let dex_share_0 = match currency_id_0 {
			CurrencyId::Token(symbol) => DexShare::Token(symbol),
			CurrencyId::Erc20(address) => DexShare::Erc20(address),
			CurrencyId::ForeignAsset(foreign_asset_id) => DexShare::ForeignAsset(foreign_asset_id),
			// Unsupported
			CurrencyId::DexShare(..) => return None,
		};
		let dex_share_1 = match currency_id_1 {
			CurrencyId::Token(symbol) => DexShare::Token(symbol),
			CurrencyId::Erc20(address) => DexShare::Erc20(address),
			CurrencyId::ForeignAsset(foreign_asset_id) => DexShare::ForeignAsset(foreign_asset_id),
			// Unsupported
			CurrencyId::DexShare(..) => return None,
		};
		Some(CurrencyId::DexShare(dex_share_0, dex_share_1))
	}

	pub fn erc20_address(&self) -> Option<EvmAddress> {
		match self {
			CurrencyId::Erc20(address) => Some(*address),
			CurrencyId::Token(_) => EvmAddress::try_from(*self).ok(),
			_ => None,
		}
	}
}

impl From<DexShare> for u32 {
	fn from(val: DexShare) -> u32 {
		let mut bytes = [0u8; 4];
		match val {
			DexShare::Token(token) => {
				bytes[3] = token.into();
			}
			DexShare::Erc20(address) => {
				// Use first 4 non-zero bytes as u32 to the mapping between u32 and evm address.
				// Take the first 4 non-zero bytes, if it is less than 4, add 0 to the left.
				let is_zero = |&&d: &&u8| -> bool { d == 0 };
				let leading_zeros = address.as_bytes().iter().take_while(is_zero).count();
				let index = if leading_zeros > 16 { 16 } else { leading_zeros };
				bytes[..].copy_from_slice(&address[index..index + 4][..]);
			}
			DexShare::ForeignAsset(foreign_asset_id) => {
				bytes[2..].copy_from_slice(&foreign_asset_id.to_be_bytes());
			}
		}
		u32::from_be_bytes(bytes)
	}
}

impl Into<CurrencyId> for DexShare {
	fn into(self) -> CurrencyId {
		match self {
			DexShare::Token(token) => CurrencyId::Token(token),
			DexShare::Erc20(address) => CurrencyId::Erc20(address),
			DexShare::ForeignAsset(foreign_asset_id) => CurrencyId::ForeignAsset(foreign_asset_id),
		}
	}
}

/// H160 CurrencyId Type enum
#[derive(
	Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TryFromPrimitive, IntoPrimitive, TypeInfo,
)]
#[repr(u8)]
pub enum CurrencyIdType {
	Token = 1, // 0 is prefix of precompile and predeploy
	DexShare,
	ForeignAsset,
	FiatCurrency,
}

#[derive(
	Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TryFromPrimitive, IntoPrimitive, TypeInfo,
)]
#[repr(u8)]
pub enum DexShareType {
	Token,
	Erc20,
	ForeignAsset,
}

impl Into<DexShareType> for DexShare {
	fn into(self) -> DexShareType {
		match self {
			DexShare::Token(_) => DexShareType::Token,
			DexShare::Erc20(_) => DexShareType::Erc20,
			DexShare::ForeignAsset(_) => DexShareType::ForeignAsset,
		}
	}
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub enum AssetIds {
	Erc20(EvmAddress),
	ForeignAssetId(ForeignAssetId),
	NativeAssetId(CurrencyId),
	FiatCurrencyId,(FiatCurrencyId),
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct AssetMetadata<Balance> {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub minimal_balance: Balance,
}
