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
				// Setheum Network LPs
				Token {
					symbol: "LP_DNAR_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(DNAR), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_SDEX_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SDEX), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_AEDJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(AEDJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_AUDJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(AUDJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_BRLJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(BRLJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_CADJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(CADJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_CHFJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(CHFJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_CLPJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(CLPJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_CNYJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(CNYJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_COPJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(COPJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_EURJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(EURJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_GBPJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(GBPJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_HKDJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(HKDJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_HUFJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(HUFJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_IDRJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(IDRJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_JPYJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JPYJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_KESJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(KESJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_KRWJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(KRWJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_KZTJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(KZTJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_MXNJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(MXNJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_MYRJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(MYRJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_NGNJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(NGNJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_NOKJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(NOKJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_NZDJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(NZDJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_PENJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(PENJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_PHPJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(PHPJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_PKRJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(PKRJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_PLNJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(PLNJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_QARJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(QARJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_RONJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(RONJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_RUBJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(RUBJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_SARJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SARJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_SEKJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SEKJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_SGDJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SGDJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_THBJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(THBJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_TRYJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(TRYJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_TWDJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(TWDJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_TZSJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(TZSJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_USDJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(USDJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_ZARJ_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(ZARJ), DexShare::Token(SETT))).unwrap(),
				},
				Token {
					symbol: "LP_RENBTC_SETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(RENBTC), DexShare::Token(SETT))).unwrap(),
				},

				// Neom Network LPs
				Token {
					symbol: "LP_NEOM_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(NEOM), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_HALAL_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(HALAL), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JAED_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JAED), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JAUD_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JAUD), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JBRL_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JBRL), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JCAD_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JCAD), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JCHF_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JCHF), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JCLP_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JCLP), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JCNY_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JCNY), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JCOP_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JCOP), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JEUR_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JEUR), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JGBP_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JGBP), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JHKD_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JHKD), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JHUF_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JHUF), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JIDR_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JIDR), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JJPY_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JJPY), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JKES_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JKES), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JKRW_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JKRW), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JKZT_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JKZT), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JMXN_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JMXN), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JMYR_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JMYR), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JNGN_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JNGN), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JNOK_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JNOK), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JNZD_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JNZD), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JPEN_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JPEN), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JPHP_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JPHP), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JPKR_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JPKR), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JPLN_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JPLN), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JQAR_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JQAR), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JRON_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JRON), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JRUB_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JRUB), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JSAR_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JSAR), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JSEK_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JSEK), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JSGD_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JSGD), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JTHB_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JTHB), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JTRY_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JTRY), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JTWD_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JTWD), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JTZS_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JTZS), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JUSD_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JUSD), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_JZAR_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(JZAR), DexShare::Token(NSETT))).unwrap(),
				},
				Token {
					symbol: "LP_RENBTC_NSETT".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(RENBTC), DexShare::Token(NSETT))).unwrap(),
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
		DNAR("Setheum Dinar", 10) = 0, // could consider having 12 decimals too.
		SDEX("SettinDex", 10) = 1, // could consider having 12 decimals too.
		SETT("Setter", 12) = 2,
		// SettCurrencies
		USDJ("Setheum US Dollar", 12) = 3,
		EURJ("Setheum Euro", 12) = 4,
		GBPJ("Setheum Pound Sterling", 12) = 5,
		JPYJ("Setheum Japanese Yen", 12) = 6,
		CADJ("Setheum Canadian Dollar", 12) = 7,
 		AUDJ("Setheum Australian Dollar", 12) = 8,
		BRLJ("Setheum Brazilian Real", 12) = 9,
		CHFJ("Setheum Swiss Franc", 12) = 10,
 		SARJ("Setheum Saudi Riyal", 12) = 11,
 		SGDJ("Setheum Singapore Dollar", 12) = 12,
		/// Neom Network >---------------------->>
		NEOM("Neom", 10) = 128,
		HALAL("HalalSwap", 10) = 129,
		NSETT("Neom Setter", 12) = 130,
		// SettCurrencies
		JUSD("Neom US Dollar", 12) = 131,
		JEUR("Neom Euro", 12) = 132,
		JGBP("Neom Pound Sterling", 12) = 133,
		JJPY("Neom Japanese Yen", 12) = 134,
		JCAD("Neom Canadian Dollar", 12) = 135,
 		JAUD("Neom Australian Dollar", 12) = 136,
		JBRL("Neom Brazilian Real", 12) = 137,
		JCHF("Neom Swiss Franc", 12) = 138,
 		JSAR("Neom Saudi Riyal", 12) = 139,
 		JSGD("Neom Singapore Dollar", 12) = 140,
		// Foreign System Currencies
		RENBTC("Ren Bitcoin", 8) = 141,
		/// Fiat Currencies as Pegs - only for price feed
 		AUD("Fiat Australian Dollar", 12) = 172,
		BRL("Fiat Brazilian Real", 12) = 173,
		CAD("Fiat Canadian Dollar", 12) = 174,
		CHF("Fiat Swiss Franc", 12) = 175,
		EUR("Fiat Euro", 12) =176,
		GBP("Fiat Pound Sterling", 12) = 177,
		JPY("Fiat Japanese Yen", 12) = 178,
 		SAR("Fiat Saudi Riyal", 12) = 179,
 		SGD("Fiat Singapore Dollar", 12) = 180,
		USD("Fiat US Dollar", 12) = 181,
		KWD("Fiat Kuwaiti Dinar", 12) = 182,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		JOD("Fiat Jordanian Dinar", 12) = 183,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		BHD("Fiat Bahraini Dirham", 12) = 184,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		KYD("Fiat Cayman Islands Dollar", 12) = 185,	// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		OMR("Fiat Omani Riyal", 12) = 186,				// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		GIP("Fiat Gibraltar Pound", 12) = 187,			// part of the Setter pegs, not having single settcurrencies they peg like the rest of the fiats here.
		/// Ends at 255
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
