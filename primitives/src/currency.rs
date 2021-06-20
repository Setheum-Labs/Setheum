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

#![allow(clippy::from_over_into)] 

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
		// TODO: Update!
		/// Setheum Network >---------------------->>
		DNAR("Setheum Dinar", 10) = 0,
		SDEX("SettinDex", 10) = 1,
		SETT("Setter", 12) = 2,
		// SettCurrencies
		USDJ("Setheum US Dollar", 12) = 3,
		EURJ("Setheum Euro", 12) = 4,
		JPYJ("Setheum Japanese Yen", 12) = 5,
		GBPJ("Setheum Pound Sterling", 12) = 6,
		CADJ("Setheum Canadian Dollar", 12) = 7,
		HKDJ("Setheum HongKong Dollar", 12) = 8,
		TWDJ("Setheum Taiwan Dollar", 12) = 9,
		BRLJ("Setheum Brazilian Real", 12) = 10,
		CHFJ("Setheum Swiss Franc", 12) = 11,
		RUBJ("Setheum Russian Rubble", 12) = 12,
		THBJ("Setheum Thai Baht", 12) = 13,
		MXNJ("Setheum Mexican Peso", 12) = 14,
 		SARJ("Setheum Saudi Riyal", 12) = 15,
 		SGDJ("Setheum Singapore Dollar", 12) = 16,
 		SEKJ("Setheum Swedish Krona", 12) = 17,
		MYRJ("Setheum Malaysian Ringgit", 12) = 18,
		IDRJ("Setheum Indonesian Rupiah", 12) = 19,
 		NGNJ("Setheum Nigerian Naira", 12) = 20,
 		PKRJ("Setheum Pakistani Rupee", 12) = 21,
		AEDJ("Setheum Emirati Dirham", 12) = 22,
		NOKJ("Setheum Norwegian Krone", 12) = 23,
		ZARJ("Setheum S.African Rand", 12) = 24,
		NZDJ("Setheum NewZealand Dollar ", 12) = 26,
		COPJ("Setheum Colombian Peso", 12) = 27,
		CLPJ("Setheum Chilean Peso", 12) = 29,
		PHPJ("Setheum Philippine Peso", 12) = 30,
		HUFJ("Setheum Hungarian Forint", 12) = 31,
		TRYJ("Setheum Turkish Lira", 12) = 33,
 		AUDJ("Setheum Australian Dollar", 12) = 34,
 		KESJ("Setheum Kenyan Shilling", 12) = 35,
 		KRWJ("Setheum S.Korean Won", 12) = 39,
		TZSJ("Setheum Tanzanian Shilling", 12) = 46,
		ARSJ("Setheum Argentine Peso", 12) = 55,
		RONJ("Setheum Romanian Leu", 12) = 56,
		PLNJ("Setheum Polish Zloty", 12) = 58,

		
		/// Neom Network >---------------------->>
		NEOM("Neom", 10) = 128,
		HALAL("HalalSwap", 10) = 129,
		NSETT("Neom Setter", 12) = 130,
		// SettCurrencies
		JUSD("Neom US Dollar", 12) = 131,
		JEUR("Neom Euro", 12) = 132,
		JJPY("Neom Japanese Yen", 12) = 133,
		JGBP("Neom Pound Sterling", 12) = 134,
		JCAD("Neom Canadian Dollar", 12) = 135,
		JHKD("Neom HongKong Dollar", 12) = 136,
		JTWD("Neom Taiwan Dollar", 12) = 137,
		JBRL("Neom Brazilian Real", 12) = 138,
		JCHF("Neom Swiss Franc", 12) = 139,
		JRUB("Neom Russian Rubble", 12) = 140,
		JTHB("Neom Thai Baht", 12) = 141,
		JMXN("Neom Mexican Peso", 12) = 142,
 		JSAR("Neom Saudi Riyal", 12) = 143,
 		JSGD("Neom Singapore Dollar", 12) = 144,
 		JSEK("Neom Swedish Krona", 12) = 145,
		JMYR("Neom Malaysian Ringgit", 12) = 146,
		JIDR("Neom Indonesian Rupiah", 12) = 147,
 		JNGN("Neom Nigerian Naira", 12) = 148,
 		JPKR("Neom Pakistani Rupee", 12) = 149,
		JAED("Neom Emirati Dirham", 12) = 150,
		JNOK("Neom Norwegian Krone", 12) = 151,
		JZAR("Neom S.African Rand", 12) = 152,
		JNZD("Neom NewZealand Dollar ", 12) = 154,
		JCOP("Neom Colombian Peso", 12) = 155,
		JCLP("Neom Chilean Peso", 12) = 157,
		JPHP("Neom Philippine Peso", 12) = 158,
		JHUF("Neom Hungarian Forint", 12) = 159,
		JTRY("Neom Turkish Lira", 12) = 161,
 		JAUD("Neom Australian Dollar", 12) = 162,
 		JKES("Neom Kenyan Shilling", 12) = 163,
 		JKRW("Neom S.Korean Won", 12) = 167,
		JTZS("Neom TZ Shilling", 12) = 174,
		JPLN("Neom Polish Zloty", 12) = 177,
		JARS("Neom Argentine Peso", 12) = 183,
		JRON("Neom Romanian Leu", 12) = 184,
		JPLN("Neom Polish Zloty", 12) = 185,


		/// Fiat Currencies as Pegs >---------------------->>
		/// Fiat - only for price feed
		/// 
		USD("US Dollar", 12) = 246,
		EUR("Euro", 12) = 247,
		JPY("Japanese Yen", 12) = 248,
		GBP("Pound Sterling", 12) = 249,
		CAD("Canadian Dollar", 12) = 250,
		HKD("HongKong Dollar", 12) = 251,
		TWD("Taiwan Dollar", 12) = 252,
		BRL("Brazilian Real", 12) = 253,
		CHF("Swiss Franc", 12) = 254,
		RUB("Russian Rubble", 12) = 255,
		THB("Thai Baht", 12) = 256,
		MXN("Mexican Peso", 12) = 257,
 		SAR("Saudi Riyal", 12) = 258,
 		SGD("Singapore Dollar", 12) = 259,
 		SEK("Swedish Krona", 12) = 260,
		MYR("Malaysian Ringgit", 12) = 261,
		IDR("Indonesian Rupiah", 12) = 262,
 		NGN("Nigerian Naira", 12) = 263,
 		PKR("Pakistani Rupee", 12) = 264,
		AED("Emirati Dirham", 12) = 265,
		NOK("Norwegian Krone", 12) = 266,
		ZAR("S.African Rand", 12) = 267,
		CZK("Czech Koruna", 12) = 268,
		NZD("NewZealand Dollar ", 12) = 269,
		COP("Colombian Peso", 12) = 270,
		KWD("Kuwaiti Dinar", 12) = 271,
		CLP("Chilean Peso", 12) = 272,
		PHP("Philippine Peso", 12) = 273,
		HUF("Hungarian Forint", 12) = 274,
		JOD("Jordanian Dinar", 12) = 275,
		TRY("Turkish Lira", 12) = 276,
 		AUD("Australian Dollar", 12) = 278,
 		KES("Kenyan Shilling", 12) = 279,
 		BHD("Bahraini Dinar", 12) = 280,
		BWP("Botswanan Pula", 12) = 2081,
		INR("Indian Rupee", 12) = 282,
 		KRW("S.Korean Won", 12) = 283,
 		SCR("Seychellois Rupee", 12) = 284,
		ZMW("Zambian Kwacha", 12) = 285,
		GHS("Ghanaian Cedi", 12) = 286,
		AOA("Angolan Kwanza", 12) = 287,
		DZD("Algerian Dinar", 12) = 288,
		ETB("Ethiopian Birr", 12) = 289,
		TZS("TZ Shilling", 12) = 290,
		CFA("CFA Franc", 12) = 291,
		AZN("Azerbaijani Manat", 12) = 292,
		PLN("Polish Zloty", 12) = 293,
		OMR("Omani Riyal", 12) = 294,
		TND("Tunisian Dinar", 12) = 295,
		MAD("Moroccan Dirham", 12) = 296,
		HRK("Croatian Kuna", 12) = 297,
		BGN("Bulgarian Lev", 12) = 298,
		DKK("Danish Krone", 12) = 299,
		ARS("Argentine Peso", 12) = 300,
		RON("Romanian Leu", 12) = 301,
		BAM("Bosnian Mark", 12) = 302,
		PLN("Polish Zloty", 12) = 303,
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
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(TokenSymbol),
	DexShare(DexShare, DexShare),
}

impl CurrencyId {
	pub fn is_token_currency_id(&self) -> bool {
		matches!(self, CurrencyId::Token(_))
	}

	pub fn is_dex_share_currency_id(&self) -> bool {
		matches!(self, CurrencyId::DexShare(_, _))
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
			CurrencyId::Token(symbol) => {
				DexShare::Token(symbol)
			}
			_ => return None,
		};
		let token_symbol_1 = match currency_id_1 {
			CurrencyId::Token(symbol) => {
				DexShare::Token(symbol)
			}
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
		}
		u32::from_be_bytes(bytes)
	}
}

impl Into<CurrencyId> for DexShare {
	fn into(self) -> CurrencyId {
		match self {
			DexShare::Token(token) => CurrencyId::Token(token),
		}
	}
}
