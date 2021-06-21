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
		DNAR("Setheum Dinar", 10) = 0,
		SDEX("SettinDex", 10) = 1,
		SETT("Setter", 12) = 2,
		// SettCurrencies (Alphabetical Order)
		AEDJ("Setheum UAE Emirati Dirham", 12) = 3,
		ARSJ("Setheum Argentine Peso", 12) = 4,
 		AUDJ("Setheum Australian Dollar", 12) = 5,
		BRLJ("Setheum Brazilian Real", 12) = 6,
		CADJ("Setheum Canadian Dollar", 12) = 7,
		CHFJ("Setheum Swiss Franc", 12) = 8,
		CLPJ("Setheum Chilean Peso", 12) = 9,
		CNYJ("Setheum Japanese Yen", 12) = 10,
		COPJ("Setheum Colombian Peso", 12) = 11,
		EURJ("Setheum Euro", 12) =12,
		GBPJ("Setheum Pound Sterling", 12) = 13,
		HKDJ("Setheum HongKong Dollar", 12) = 14,
		HUFJ("Setheum Hungarian Forint", 12) = 15,
		IDRJ("Setheum Indonesian Rupiah", 12) = 16,
		IRRJ("Setheum Iranian Riyal", 12) = 17,
		JPYJ("Setheum Japanese Yen", 12) = 18,
 		KESJ("Setheum Kenyan Shilling", 12) = 19,
 		KRWJ("Setheum South Korean Won", 12) = 20,
 		KZTJ("Setheum Kzakhstani Tenge", 12) = 21,
		MXNJ("Setheum Mexican Peso", 12) = 22,
		MYRJ("Setheum Malaysian Ringgit", 12) = 23,
 		NGNJ("Setheum Nigerian Naira", 12) = 24,
		NOKJ("Setheum Norwegian Krone", 12) = 25,
		NZDJ("Setheum New Zealand Dollar ", 12) = 26,
		PENJ("Setheum Peruvian Sol", 12) = 27,
		PHPJ("Setheum Philippine Peso", 12) = 28,
 		PKRJ("Setheum Pakistani Rupee", 12) = 29,
		PLNJ("Setheum Polish Zloty", 12) = 30,
		QARJ("Setheum Qatari Riyal", 12) = 31,
		RONJ("Setheum Romanian Leu", 12) = 32,
		RUBJ("Setheum Russian Rubble", 12) = 33,
 		SARJ("Setheum Saudi Riyal", 12) = 34,
 		SEKJ("Setheum Swedish Krona", 12) = 35,
 		SGDJ("Setheum Singapore Dollar", 12) = 36,
		THBJ("Setheum Thai Baht", 12) = 37,
		TRYJ("Setheum Turkish Lira", 12) = 38,
		TWDJ("Setheum Taiwan Dollar", 12) = 39,
		TZSJ("Setheum Tanzanian Shilling", 12) = 40,
		UAHJ("Setheum Ukranian Hryvnia", 12) = 41,
		USDJ("Setheum US Dollar", 12) = 42,
		ZARJ("Setheum South African Rand", 12) = 43,
		/// Ends at 85 (42 places left yet reserved)

		/// Neom Network >---------------------->>
		/// Starts from 86 (85 places available)
		NEOM("Neom", 10) = 86,
		HALAL("HalalSwap", 10) = 87,
		NSETT("Neom Setter", 12) = 88,
		// SettCurrencies (Alphabetical Order)
		JAED("Neom UAE Emirati Dirham", 12) = 3,
		JARS("Neom Argentine Peso", 12) = 4,
 		JAUD("Neom Australian Dollar", 12) = 5,
		JBRL("Neom Brazilian Real", 12) = 6,
		JCAD("Neom Canadian Dollar", 12) = 7,
		JCHF("Neom Swiss Franc", 12) = 8,
		JCLP("Neom Chilean Peso", 12) = 9,
		JCNY("Neom Japanese Yen", 12) = 10,
		JCOP("Neom Colombian Peso", 12) = 11,
		JEUR("Neom Euro", 12) =12,
		JGBP("Neom Pound Sterling", 12) = 13,
		JHKD("Neom HongKong Dollar", 12) = 14,
		JHUF("Neom Hungarian Forint", 12) = 15,
		JIDR("Neom Indonesian Rupiah", 12) = 16,
		JIRR("Neom Iranian Riyal", 12) = 17,
		JJPY("Neom Japanese Yen", 12) = 18,
 		JKES("Neom Kenyan Shilling", 12) = 19,
 		JKRW("Neom South Korean Won", 12) = 20,
 		JKZT("Neom Kzakhstani Tenge", 12) = 21,
		JMXN("Neom Mexican Peso", 12) = 22,
		JMYR("Neom Malaysian Ringgit", 12) = 23,
 		JNGN("Neom Nigerian Naira", 12) = 24,
		JNOK("Neom Norwegian Krone", 12) = 25,
		JNZD("Neom New Zealand Dollar ", 12) = 26,
		JPEN("Neom Peruvian Sol", 12) = 27,
		JPHP("Neom Philippine Peso", 12) = 28,
 		JPKR("Neom Pakistani Rupee", 12) = 29,
		JPLN("Neom Polish Zloty", 12) = 30,
		JQAR("Neom Qatari Riyal", 12) = 31,
		JRON("Neom Romanian Leu", 12) = 32,
		JRUB("Neom Russian Rubble", 12) = 33,
 		JSAR("Neom Saudi Riyal", 12) = 34,
 		JSEK("Neom Swedish Krona", 12) = 35,
 		JSGD("Neom Singapore Dollar", 12) = 36,
		JTHB("Neom Thai Baht", 12) = 37,
		JTRY("Neom Turkish Lira", 12) = 38,
		JTWD("Neom Taiwan Dollar", 12) = 39,
		JTZS("Neom Tanzanian Shilling", 12) = 40,
		JUAH("Neom Ukranian Hryvnia", 12) = 41,
		JUSD("Neom US Dollar", 12) = 42,
		JZAR("Neom South African Rand", 12) = 43,
		/// Ends at 170 (42 places left yet reserved)

		/// Fiat Currencies as Pegs
		/// Fiat Currencies - only for price feed (Alphabetical Order)
		/// Starts from 171 (85 places available)
		AED("Fiat UAE Emirati Dirham", 12) = 171,
		ARS("Fiat Argentine Peso", 12) = 172,
 		AUD("Fiat Australian Dollar", 12) = 173,
		BRL("Fiat Brazilian Real", 12) = 174,
		CAD("Fiat Canadian Dollar", 12) = 175,
		CHF("Fiat Swiss Franc", 12) = 176,
		CLP("Fiat Chilean Peso", 12) = 177,
		CNY("Fiat Japanese Yen", 12) = 178,
		COP("Fiat Colombian Peso", 12) = 179,
		EUR("Fiat Euro", 12) =180,
		GBP("Fiat Pound Sterling", 12) = 181,
		HKD("Fiat HongKong Dollar", 12) = 182,
		HUF("Fiat Hungarian Forint", 12) = 183,
		IDR("Fiat Indonesian Rupiah", 12) = 184,
		IRR("Fiat Iranian Riyal", 12) = 185,
		JPY("Fiat Japanese Yen", 12) = 186,
 		KES("Fiat Kenyan Shilling", 12) = 187,
 		KRW("Fiat South Korean Won", 12) = 188,
 		KZT("Fiat Kzakhstani Tenge", 12) = 189,
		MXN("Fiat Mexican Peso", 12) = 190,
		MYR("Fiat Malaysian Ringgit", 12) = 191,
 		NGN("Fiat Nigerian Naira", 12) = 192,
		NOK("Fiat Norwegian Krone", 12) = 193,
		NZD("Fiat New Zealand Dollar ", 12) = 194,
		PEN("Fiat Peruvian Sol", 12) = 195,
		PHP("Fiat Philippine Peso", 12) = 196,
 		PKR("Fiat Pakistani Rupee", 12) = 197,
		PLN("Fiat Polish Zloty", 12) = 198,
		QAR("Fiat Qatari Riyal", 12) = 199,
		RON("Fiat Romanian Leu", 12) = 200,
		RUB("Fiat Russian Rubble", 12) = 201,
 		SAR("Fiat Saudi Riyal", 12) = 202,
 		SEK("Fiat Swedish Krona", 12) = 203,
 		SGD("Fiat Singapore Dollar", 12) = 204,
		THB("Fiat Thai Baht", 12) = 205,
		TRY("Fiat Turkish Lira", 12) = 206,
		TWD("Fiat Taiwan Dollar", 12) = 207,
		TZS("Fiat Tanzanian Shilling", 12) = 208,
		UAH("Fiat Ukranian Hryvnia", 12) = 209,
		USD("Fiat US Dollar", 12) = 210,
		ZAR("Fiat South African Rand", 12) = 211,
		KWD("Fiat Kuwaiti Dinar", 12) = 212,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		JOD("Fiat Jordanian Dinar", 12) = 213,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		BHD("Fiat Bahraini Dirham", 12) = 214,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		KYD("Fiat Cayman Islands Dollar", 12) = 215,	// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		OMR("Fiat Omani Riyal", 12) = 216,				// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		GIP("Fiat Gibraltar Pound", 12) = 217,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		/// Ends at 255 (38 places left yet reserved).
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
	ChainSafe(chainbridge::ResourceId),
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
			CurrencyId::ChainSafe(_) => Err(()),
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
