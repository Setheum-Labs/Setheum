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

use bstringify::bstringify;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::{
	convert::{Into, TryFrom, TryInto},
	prelude::*,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

macro_rules! create_currency_id {
    ($(#[$meta:meta])*
	$vis:vis enum TokenSymbol {
        $($(#[$vmeta:meta])* $vname:ident($deci:literal) = $val:literal,)*
    }) => {
        $(#[$meta])*
        $vis enum TokenSymbol {
            $($(#[$vmeta])* $vname = $val,)*
        }

        impl TryFrom<u8> for TokenSymbol {
            type Error = ();

            fn try_from(v: u8) -> Result<Self, Self::Error> {
                match v {
                    $($val => Ok(TokenSymbol::$vname),)*
                    _ => Err(()),
                }
            }
        }

		impl TryFrom<Vec<u8>> for CurrencyId {
			type Error = ();
			fn try_from(v: Vec<u8>) -> Result<CurrencyId, ()> {
				match v.as_slice() {
					$(bstringify!($vname) => Ok(CurrencyId::Token(TokenSymbol::$vname)),)*
					_ => Err(()),
				}
			}
		}

		impl GetDecimals for CurrencyId {
			fn decimals(&self) -> u32 {
				match self {
					$(CurrencyId::Token(TokenSymbol::$vname) => $deci,)*
					CurrencyId::DexShare(symbol_0, symbol_1) => sp_std::cmp::max(CurrencyId::Token(*symbol_0).decimals(), CurrencyId::Token(*symbol_1).decimals()),
				}
			}
		}

		$(pub const $vname: CurrencyId = CurrencyId::Token(TokenSymbol::$vname);)*

		impl TokenSymbol {
			pub fn get_info() -> Vec<(&'static str, u32)> {
				vec![
					$((stringify!($vname), $deci),)*
				]
			}
		}

    }
}

create_currency_id! {
	// Represent a Token symbol with 8 bit
	// Bit 8 : 0 for Pokladot Ecosystem, 1 for Kusama Ecosystem
	// Bit 7 : Reserved
	// Bit 6 - 1 : The token ID
	#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[repr(u8)]
	pub enum TokenSymbol {
		/// Setheum Network
		/// Make it in alphabetical order.
		DNAR("Setheum Dinar", 10) = 0,
		SDEX("SettinDex", 10) = 1,
		SETT("Setter", 12) = 2,

		USDJ("Setheum US Dollar", 12) = 3,
		EURJ("Setheum Euro", 12) = 4,
		JPYJ("Setheum Japanese Yen", 12) = 5,
		GBPJ("Setheum Pound Sterling", 12) = 6,
		CADJ("Setheum Canadian Dollar", 12) = 7,
		HKDJ("Setheum HK Dollar", 12) = 8,
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
 		NGNJ("Setheum Naira", 12) = 20,
 		PKRJ("Setheum Pakistani Rupee", 12) = 21,
		AEDJ("Setheum Emirati Dirham", 12) = 22,
		PLNJ("Setheum Polish Zloty", 12) = 23,
		NOKJ("Setheum Norwegian Krone", 12) = 24,
		ZARJ("Setheum S.African Rand", 12) = 25,
		CZKJ("Setheum Czech Koruna", 12) = 26,
		NZDJ("Setheum NZ Dollar ", 12) = 27,
		COPJ("Setheum Colombian Peso", 12) = 28,
		KWDJ("Setheum Kuwaiti Dinar", 12) = 29,
		CLPJ("Setheum Chilean Peso", 12) = 30,
		PHPJ("Setheum Philippine Peso", 12) = 31,
		HUFJ("Setheum Hungarian Forint", 12) = 32,
		JODJ("Setheum Jordanian Dinar", 12) = 33,
		TRYJ("Setheum Turkish Lira", 12) = 34,
 		AUDJ("Setheum Aussie Dollar", 12) = 35,
 		KESJ("Setheum Kenyan Shilling", 12) = 36,
 		BHDJ("Setheum Bahraini Dinar", 12) = 37,
		BWPJ("Setheum Botswanan Pula", 12) = 38,
		INRJ("Setheum Indian Rupee", 12) = 39,
 		KRWJ("Setheum S.Korean Won", 12) = 40,
 		SCRJ("Setheum Seychellois Rupee", 12) = 41,
		ZMWJ("Setheum Zambian Kwacha", 12) = 42,
		GHSJ("Setheum Ghanaian Cedi", 12) = 43,
		AOAJ("Setheum Angolan Kwanza", 12) = 44,
		DZDJ("Setheum Algerian Dinar", 12) = 45,
		ETBJ("Setheum Ethiopian Birr", 12) = 46,
		TZSJ("Setheum TZ Shilling", 12) = 47,
		CFAJ("Setheum CFA Franc", 12) = 48,
		AZNJ("Setheum Azerbaijani Manat", 12) = 49,

		/// Neom Network
		NEOM("Neom", 10) = 256,
		HALAL("HalalSwap", 10) = 257,
		NSETT("Neom Setter", 12) = 258,

		JUSD("Neom US Dollar", 12) = 259,
		JEUR("Neom Euro", 12) = 4,
		JJPY("Neom Japanese Yen", 12) = 260,
		JGBP("Neom Pound Sterling", 12) = 261,
		JCAD("Neom Canadian Dollar", 12) = 262,
		JHKD("Neom HK Dollar", 12) = 8,
		JTWD("Neom Taiwan Dollar", 12) = 263,
		JBRL("Neom Brazilian Real", 12) = 264,
		JCHF("Neom Swiss Franc", 12) = 265,
		JRUB("Neom Russian Rubble", 12) = 266,
		JTHB("Neom Thai Baht", 12) = 267,
		JMXN("Neom Mexican Peso", 12) = 268,
 		JSAR("Neom Saudi Riyal", 12) = 269,
 		JSGD("Neom Singapore Dollar", 12) = 270,
 		JSEK("Neom Swedish Krona", 12) = 271,
		JMYR("Neom Malaysian Ringgit", 12) = 272,
		JIDR("Neom Indonesian Rupiah", 12) = 273,
 		JNGN("Neom Naira", 12) = 274,
 		JPKR("Neom Pakistani Rupee", 12) = 275,
		JAED("Neom Emirati Dirham", 12) = 276,
		JPLN("Neom Polish Zloty", 12) = 277,
		JNOK("Neom Norwegian Krone", 12) = 278,
		JZAR("Neom S.African Rand", 12) = 279,
		JCZK("Neom Czech Koruna", 12) = 280,
		JNZD("Neom NZ Dollar ", 12) = 281,
		JCOP("Neom Colombian Peso", 12) = 282,
		JKWD("Neom Kuwaiti Dinar", 12) = 283
		JCLP("Neom Chilean Peso", 12) = 284,
		JPHP("Neom Philippine Peso", 12) = 285,
		JHUF("Neom Hungarian Forint", 12) = 286,
		JJOD("Neom Jordanian Dinar", 12) = 287,
		JTRY("Neom Turkish Lira", 12) = 288,
 		JAUD("Neom Aussie Dollar", 12) = 289,
 		JKES("Neom Kenyan Shilling", 12) = 290,
 		JBHD("Neom Bahraini Dinar", 12) = 291,
		JBWP("Neom Botswanan Pula", 12) = 292,
		JINR("Neom Indian Rupee", 12) = 293,
 		JKRW("Neom S.Korean Won", 12) = 294,
 		JSCR("Neom Seychellois Rupee", 12) = 295,
		JZMW("Neom Zambian Kwacha", 12) = 296,
		JGHS("Neom Ghanaian Cedi", 12) = 297,
		JAOA("Neom Angolan Kwanza", 12) = 298,
		JDZD("Neom Algerian Dinar", 12) = 299,
		JETB("Neom Ethiopian Birr", 12) = 300,
		JTZS("Neom TZ Shilling", 12) = 301,
		JCFA("Neom CFA Franc", 12) = 302,
		JAZN("Neom Azerbaijani Manat", 12) = 303,

	}
}

pub trait GetDecimals {
	fn decimals(&self) -> u32;
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(TokenSymbol),
	DexShare(TokenSymbol, TokenSymbol),
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
				Some((CurrencyId::Token(*token_symbol_0), CurrencyId::Token(*token_symbol_1)))
			}
			_ => None,
		}
	}

	pub fn join_dex_share_currency_id(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
		match (currency_id_0, currency_id_1) {
			(CurrencyId::Token(token_symbol_0), CurrencyId::Token(token_symbol_1)) => {
				Some(CurrencyId::DexShare(token_symbol_0, token_symbol_1))
			}
			_ => None,
		}
	}
}

impl TryFrom<[u8; 32]> for CurrencyId {
	type Error = ();

	fn try_from(v: [u8; 32]) -> Result<Self, Self::Error> {
		if !v.starts_with(&[0u8; 29][..]) {
			return Err(());
		}

		// token
		if v[29] == 0 && v[31] == 0 {
			return v[30].try_into().map(CurrencyId::Token);
		}

		// Dex share
		if v[29] == 1 {
			let left = v[30].try_into()?;
			let right = v[31].try_into()?;
			return Ok(CurrencyId::DexShare(left, right));
		}

		Err(())
	}
}

impl From<CurrencyId> for [u8; 32] {
	fn from(val: CurrencyId) -> Self {
		let mut bytes = [0u8; 32];
		match val {
			CurrencyId::Token(token) => {
				bytes[30] = token as u8;
			}
			CurrencyId::DexShare(left, right) => {
				bytes[29] = 1;
				bytes[30] = left as u8;
				bytes[31] = right as u8;
			}
		}
		bytes
	}
}
