// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

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
use num_enum::{IntoPrimitive, TryFromPrimitive};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{
	convert::{Into, TryFrom},
	prelude::*, vec,
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
					symbol: "LP_SETM_USDI".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SETM), DexShare::Token(USDI))).unwrap(),
				},
				Token {
					symbol: "LP_SETM_ETH".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SETM), DexShare::Token(ETH))).unwrap(),
				},
				Token {
					symbol: "LP_SETM_WBTC".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SETM), DexShare::Token(WBTC))).unwrap(),
				},
				Token {
					symbol: "LP_SETM_BNB".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SETM), DexShare::Token(BNB))).unwrap(),
				},
				Token {
					symbol: "LP_SETM_USDW".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SETM), DexShare::Token(USDW))).unwrap(),
				},
				Token {
					symbol: "LP_SETM_BUSD".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SETM), DexShare::Token(BUSD))).unwrap(),
				},
				Token {
					symbol: "LP_SETM_USDP".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SETM), DexShare::Token(USDP))).unwrap(),
				},
				Token {
					symbol: "LP_SETM_PAXG".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SETM), DexShare::Token(PAXG))).unwrap(),
				},
				// Token {
				// 	symbol: "LP_SETM_DOT".to_string(),
				// 	address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SETM), DexShare::Token(DOT))).unwrap(),
				// },
				// Token {
				// 	symbol: "LP_SETM_KSM".to_string(),
				// 	address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SETM), DexShare::Token(KSM))).unwrap(),
				// },

				// Slixon LP Tokens
				Token {
					symbol: "LP_SLIX_USDW".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SLIX), DexShare::Token(USDW))).unwrap(),
				},
				Token {
					symbol: "LP_SLIX_ETH".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SLIX), DexShare::Token(ETH))).unwrap(),
				},
				Token {
					symbol: "LP_SLIX_WBTC".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SLIX), DexShare::Token(WBTC))).unwrap(),
				},
				Token {
					symbol: "LP_SLIX_BNB".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SLIX), DexShare::Token(BNB))).unwrap(),
				},
				Token {
					symbol: "LP_SLIX_USDW".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SLIX), DexShare::Token(USDW))).unwrap(),
				},
				Token {
					symbol: "LP_SLIX_BUSD".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SLIX), DexShare::Token(BUSD))).unwrap(),
				},
				Token {
					symbol: "LP_SLIX_USDP".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SLIX), DexShare::Token(USDP))).unwrap(),
				},
				Token {
					symbol: "LP_SLIX_PAXG".to_string(),
					address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SLIX), DexShare::Token(PAXG))).unwrap(),
				},
				// Token {
				// 	symbol: "LP_SLIX_DOT".to_string(),
				// 	address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SLIX), DexShare::Token(DOT))).unwrap(),
				// },
				// Token {
				// 	symbol: "LP_SLIX_KSM".to_string(),
				// 	address: EvmAddress::try_from(CurrencyId::DexShare(DexShare::Token(SLIX), DexShare::Token(KSM))).unwrap(),
				// },
			];
			tokens.append(&mut lp_tokens);

			frame_support::assert_ok!(std::fs::write("../lib-serml/sevm/predeploy-contracts/resources/tokens.json", serde_json::to_string_pretty(&tokens).unwrap()));
		}
    }
}

create_currency_id! {
	// Represent a Token symbol with 8 bit
	#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[repr(u8)]
	pub enum TokenSymbol {
		// Native Tokens
		SETM("Setheum", 18) = 0,
		SLIX("Slixon", 18) = 1,
		USDI("InterUSD", 18) = 2,
		USDW("WestUSD", 18) = 3,
		// Foreign Tokens
		ETH("Ethereum", 18) = 4,
		WBTC("Wrapped Bitcoin", 9) = 5,
		BNB("BNB", 18) = 6,
		// Reserve DOT("Polkadot", 18) = 13,
		// Reserve KSM("Kusama", 18) = 14,
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
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum DexShare {
	Token(TokenSymbol),
	Erc20(EvmAddress),
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum CurrencyId {
	Token(TokenSymbol),
	DexShare(DexShare, DexShare),
	Erc20(EvmAddress),
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

	pub fn is_trading_pair_currency_id(&self) -> bool {
		matches!(
			self,
			CurrencyId::Token(_) | CurrencyId::Erc20(_)
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
			// Unsupported
			CurrencyId::DexShare(..) => return None,
		};
		let dex_share_1 = match currency_id_1 {
			CurrencyId::Token(symbol) => DexShare::Token(symbol),
			CurrencyId::Erc20(address) => DexShare::Erc20(address),
			// Unsupported
			CurrencyId::DexShare(..) => return None,
		};
		Some(CurrencyId::DexShare(dex_share_0, dex_share_1))
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
}

#[derive(
	Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TryFromPrimitive, IntoPrimitive, TypeInfo,
)]
#[repr(u8)]
pub enum DexShareType {
	Token,
	Erc20,
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
		}
		u32::from_be_bytes(bytes)
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

impl Into<DexShareType> for DexShare {
	fn into(self) -> DexShareType {
		match self {
			DexShare::Token(_) => DexShareType::Token,
			DexShare::Erc20(_) => DexShareType::Erc20,
		}
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
		}
	}
}
