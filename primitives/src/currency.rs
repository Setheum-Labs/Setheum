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

#![allow(clippy::from_over_into)] 

use crate::{evm::EvmAddress, *};
use bstringify::bstringify;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::{
	convert::{Into, TryFrom},
	prelude::*,
};

#[cfg(feature = "std")]
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

			let mut tokens = vec![
				$(
					Token {
						symbol: stringify!($symbol).to_string(),
						address: EvmAddress::try_from(CurrencyId::Token(TokenSymbol::$symbol)).unwrap(),
					},
				)*
			];

			let mut lp_tokens = vec![
				Token {
					symbol: "LP_DNAR_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(DNAR), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_NEOM_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(NEOM), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_DNAR_USDJ".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(DNAR), DexShare::Token(USDJ))).unwrap(),
				},
				Token {
					symbol: "LP_NEOM_JUSD".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(NEOM), DexShare::Token(JUSD))).unwrap(),
				},
				Token {
					symbol: "LP_DNAR_CNYJ".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(DNAR), DexShare::Token(CNYJ))).unwrap(),
				},
				Token {
					symbol: "LP_NEOM_JCNY".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(NEOM), DexShare::Token(JCNY))).unwrap(),
				},
				Token {
					symbol: "LP_DNAR_NGNJ".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(DNAR), DexShare::Token(NGNJ))).unwrap(),
				},
				Token {
					symbol: "LP_NEOM_JNGN".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(NEOM), DexShare::Token(JNGN))).unwrap(),
				},
			];
			tokens.append(&mut lp_tokens);

			frame_support::assert_ok!(std::fs::write("../predeploy-contracts/resources/tokens.json", serde_json::to_string_pretty(&tokens).unwrap()));
		}
    }
}

create_currency_id! {
	// Represent a Token symbol with 8 bit
	// Bit 8 : 0 for Setheum Network, 1 for Neom Network
	// Bit 7 : Reserved
	// Bit 6 - 1 : The token ID
	#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[repr(u8)]
	pub enum TokenSymbol {
		/// Setheum Network
		/// Starts from 0 (85 places available)
		DNAR("Setheum Dinar", 10) = 0, // could consider having 12 decimals too.
		SDEX("SettinDex", 10) = 1, // could consider having 12 decimals too.
		SETT("Setter", 12) = 2,
		// SettCurrencies (Alphabetical Order)
		AEDJ("Setheum UAE Emirati Dirham", 12) = 3,
 		AUDJ("Setheum Australian Dollar", 12) = 4,
		BRLJ("Setheum Brazilian Real", 12) = 5,
		CADJ("Setheum Canadian Dollar", 12) = 6,
		CHFJ("Setheum Swiss Franc", 12) = 7,
		CLPJ("Setheum Chilean Peso", 12) = 8,
		CNYJ("Setheum Japanese Yen", 12) = 9,
		COPJ("Setheum Colombian Peso", 12) = 10,
		EURJ("Setheum Euro", 12) = 11,
		GBPJ("Setheum Pound Sterling", 12) = 12,
		HKDJ("Setheum HongKong Dollar", 12) = 13,
		HUFJ("Setheum Hungarian Forint", 12) = 14,
		IDRJ("Setheum Indonesian Rupiah", 12) = 15,
		JPYJ("Setheum Japanese Yen", 12) = 16,
 		KESJ("Setheum Kenyan Shilling", 12) = 17,
 		KRWJ("Setheum South Korean Won", 12) = 18,
 		KZTJ("Setheum Kzakhstani Tenge", 12) = 19,
		MXNJ("Setheum Mexican Peso", 12) = 20,
		MYRJ("Setheum Malaysian Ringgit", 12) = 21,
 		NGNJ("Setheum Nigerian Naira", 12) = 22,
		NOKJ("Setheum Norwegian Krone", 12) = 23,
		NZDJ("Setheum New Zealand Dollar ", 12) = 24,
		PENJ("Setheum Peruvian Sol", 12) = 25,
		PHPJ("Setheum Philippine Peso", 12) = 26,
 		PKRJ("Setheum Pakistani Rupee", 12) = 27,
		PLNJ("Setheum Polish Zloty", 12) = 28,
		QARJ("Setheum Qatari Riyal", 12) = 29,
		RONJ("Setheum Romanian Leu", 12) = 30,
		RUBJ("Setheum Russian Rubble", 12) = 31,
 		SARJ("Setheum Saudi Riyal", 12) = 32,
 		SEKJ("Setheum Swedish Krona", 12) = 33,
 		SGDJ("Setheum Singapore Dollar", 12) = 34,
		THBJ("Setheum Thai Baht", 12) = 35,
		TRYJ("Setheum Turkish Lira", 12) = 36,
		TWDJ("Setheum New Taiwan Dollar", 12) = 37,
		TZSJ("Setheum Tanzanian Shilling", 12) = 38,
		USDJ("Setheum US Dollar", 12) = 39,
		ZARJ("Setheum South African Rand", 12) = 40,
		// Foreign System Currencies (Alphabetical Order)
		RENBTC("renBTC", 8) = 41,
		/// Ends at 85 (41 places left yet reserved for Setheum Network)

		/// Neom Network >---------------------->>
		/// Starts from 85 (85 places available)
		NEOM("Neom", 10) = 86,
		HALAL("HalalSwap", 10) = 87,
		NSETT("Neom Setter", 12) = 88,
		// SettCurrencies (Alphabetical Order)
		JAED("Neom UAE Emirati Dirham", 12) = 89,
 		JAUD("Neom Australian Dollar", 12) = 90,
		JBRL("Neom Brazilian Real", 12) = 91,
		JCAD("Neom Canadian Dollar", 12) = 92,
		JCHF("Neom Swiss Franc", 12) = 93,
		JCLP("Neom Chilean Peso", 12) = 94,
		JCNY("Neom Japanese Yen", 12) = 95,
		JCOP("Neom Colombian Peso", 12) = 96,
		JEUR("Neom Euro", 12) =97,
		JGBP("Neom Pound Sterling", 12) = 98,
		JHKD("Neom HongKong Dollar", 12) = 99,
		JHUF("Neom Hungarian Forint", 12) = 100,
		JIDR("Neom Indonesian Rupiah", 12) = 101,
		JJPY("Neom Japanese Yen", 12) = 102,
 		JKES("Neom Kenyan Shilling", 12) = 103,
 		JKRW("Neom South Korean Won", 12) = 104,
 		JKZT("Neom Kzakhstani Tenge", 12) = 105,
		JMXN("Neom Mexican Peso", 12) = 106,
		JMYR("Neom Malaysian Ringgit", 12) = 107,
 		JNGN("Neom Nigerian Naira", 12) = 108,
		JNOK("Neom Norwegian Krone", 12) = 109,
		JNZD("Neom New Zealand Dollar ", 12) = 110,
		JPEN("Neom Peruvian Sol", 12) = 111,
		JPHP("Neom Philippine Peso", 12) = 112,
 		JPKR("Neom Pakistani Rupee", 12) = 113,
		JPLN("Neom Polish Zloty", 12) = 114,
		JQAR("Neom Qatari Riyal", 12) = 115,
		JRON("Neom Romanian Leu", 12) = 116,
		JRUB("Neom Russian Rubble", 12) = 117,
 		JSAR("Neom Saudi Riyal", 12) = 118,
 		JSEK("Neom Swedish Krona", 12) = 119,
 		JSGD("Neom Singapore Dollar", 12) = 120,
		JTHB("Neom Thai Baht", 12) = 121,
		JTRY("Neom Turkish Lira", 12) = 122,
		JTWD("Neom New Taiwan Dollar", 12) = 123,
		JTZS("Neom Tanzanian Shilling", 12) = 124,
		JUSD("Neom US Dollar", 12) = 125,
		JZAR("Neom South African Rand", 12) = 126,
		/// Ends at 170 (41 places left yet reserved for Neom Network)

		/// Fiat Currencies as Pegs
		/// Fiat Currencies - only for price feed (Alphabetical Order)
		/// Starts from 171 (85 places available)
		AED("Fiat UAE Emirati Dirham", 12) = 171,
 		AUD("Fiat Australian Dollar", 12) = 172,
		BRL("Fiat Brazilian Real", 12) = 173,
		CAD("Fiat Canadian Dollar", 12) = 174,
		CHF("Fiat Swiss Franc", 12) = 175,
		CLP("Fiat Chilean Peso", 12) = 176,
		CNY("Fiat Japanese Yen", 12) = 177,
		COP("Fiat Colombian Peso", 12) = 178,
		EUR("Fiat Euro", 12) =179,
		GBP("Fiat Pound Sterling", 12) = 180,
		HKD("Fiat HongKong Dollar", 12) = 181,
		HUF("Fiat Hungarian Forint", 12) = 182,
		IDR("Fiat Indonesian Rupiah", 12) = 183,
		JPY("Fiat Japanese Yen", 12) = 184,
 		KES("Fiat Kenyan Shilling", 12) = 185,
 		KRW("Fiat South Korean Won", 12) = 186,
 		KZT("Fiat Kzakhstani Tenge", 12) = 187,
		MXN("Fiat Mexican Peso", 12) = 188,
		MYR("Fiat Malaysian Ringgit", 12) = 189,
 		NGN("Fiat Nigerian Naira", 12) = 190,
		NOK("Fiat Norwegian Krone", 12) = 191,
		NZD("Fiat New Zealand Dollar ", 12) = 192,
		PEN("Fiat Peruvian Sol", 12) = 193,
		PHP("Fiat Philippine Peso", 12) = 194,
 		PKR("Fiat Pakistani Rupee", 12) = 195,
		PLN("Fiat Polish Zloty", 12) = 196,
		QAR("Fiat Qatari Riyal", 12) = 197,
		RON("Fiat Romanian Leu", 12) = 198,
		RUB("Fiat Russian Rubble", 12) = 199,
 		SAR("Fiat Saudi Riyal", 12) = 200,
 		SEK("Fiat Swedish Krona", 12) = 201,
 		SGD("Fiat Singapore Dollar", 12) = 202,
		THB("Fiat Thai Baht", 12) = 203,
		TRY("Fiat Turkish Lira", 12) = 204,
		TWD("Fiat New Taiwan Dollar", 12) = 205,
		TZS("Fiat Tanzanian Shilling", 12) = 206,
		USD("Fiat US Dollar", 12) = 207,
		ZAR("Fiat South African Rand", 12) = 208,
		KWD("Fiat Kuwaiti Dinar", 12) = 209,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		JOD("Fiat Jordanian Dinar", 12) = 210,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		BHD("Fiat Bahraini Dirham", 12) = 211,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		KYD("Fiat Cayman Islands Dollar", 12) = 212,	// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		OMR("Fiat Omani Riyal", 12) = 213,				// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		GIP("Fiat Gibraltar Pound", 12) = 214,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		/// Ends at 255 (39 places left yet reserved for Fiat-Pegs).
		/// 
		/// A total of 255 (with a total of 121 places left yet reserved)
	}
}

pub trait TokenInfo {
	fn currency_id(&self) -> Option<u8>;
	fn name(&self) -> Option<&str>;
	fn symbol(&self) -> Option<&str>;
	fn decimals(&self) -> Option<u8>;
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DexShare {
	Token(TokenSymbol),
	Erc20(EvmAddress),
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(TokenSymbol),
	DexShare(DexShare, DexShare),
	Erc20(EvmAddress),
	ChainBridge(chainbridge::ResourceId),
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

	pub fn split_dex_share_currency_id(&self) -> Option<(Self, Self)> {
		match self {
			CurrencyId::DexShare(token_symbol_0, token_symbol_1) => {
				let symbol_0: CurrencyId = (*token_symbol_0).into();
				let symbol_1: CurrencyId = (*token_symbol_1).into();
				Some((symbol_0, symbol_1))
			}
			_ => None,
		}
	}

	pub fn join_dex_share_currency_id(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
		let token_symbol_0 = match currency_id_0 {
			CurrencyId::Token(symbol) => DexShare::Token(symbol),
			CurrencyId::Erc20(address) => DexShare::Erc20(address),
			_ => return None,
		};
		let token_symbol_1 = match currency_id_1 {
			CurrencyId::Token(symbol) => DexShare::Token(symbol),
			CurrencyId::Erc20(address) => DexShare::Erc20(address),
			_ => return None,
		};
		Some(CurrencyId::DexShare(token_symbol_0, token_symbol_1))
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
				let is_zero = |&&d: &&u8| -> bool { d == 0 };
				let leading_zeros = address.as_bytes().iter().take_while(is_zero).count();
				let index = if leading_zeros > 16 { 16 } else { leading_zeros };
				bytes[..].copy_from_slice(&address[index..index + 4][..]);
			}
		}
		u32::from_be_bytes(bytes)
	}
}

/// Generate the EvmAddress from CurrencyId so that evm contracts can call the erc20 contract.
impl TryFrom<CurrencyId> for EvmAddress {
	type Error = ();

	fn try_from(val: CurrencyId) -> Result<Self, Self::Error> {
		match val {
			CurrencyId::Token(_) => Ok(EvmAddress::from_low_u64_be(
				MIRRORED_TOKENS_ADDRESS_START | u64::from(val.currency_id().unwrap()),
			)),
			CurrencyId::DexShare(token_symbol_0, token_symbol_1) => {
				let symbol_0 = match token_symbol_0 {
					DexShare::Token(token) => CurrencyId::Token(token).currency_id().ok_or(()),
					DexShare::Erc20(_) => Err(()),
				}?;
				let symbol_1 = match token_symbol_1 {
					DexShare::Token(token) => CurrencyId::Token(token).currency_id().ok_or(()),
					DexShare::Erc20(_) => Err(()),
				}?;

				let mut prefix = EvmAddress::default();
				prefix[0..H160_PREFIX_DEXSHARE.len()].copy_from_slice(&H160_PREFIX_DEXSHARE);
				Ok(prefix | EvmAddress::from_low_u64_be(u64::from(symbol_0) << 32 | u64::from(symbol_1)))
			}
			CurrencyId::Erc20(address) => Ok(address),
			CurrencyId::ChainBridge(_) => Err(()),
		}
	}
}

impl Into<CurrencyId> for DexShare {
	fn into(self) -> CurrencyId {
		match self {
			DexShare::Token(token) => CurrencyId::Token(token),
			DexShare::Erc20(address) => CurrencyId::Erc20(address),
		}
	}
}
