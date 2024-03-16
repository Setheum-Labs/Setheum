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

use super::*;
use crate::evm::{
	decode_gas_limit, decode_gas_price, is_system_contract, EvmAddress, MAX_GAS_LIMIT_CC,
	SYSTEM_CONTRACT_ADDRESS_PREFIX,
};
use frame_support::assert_ok;
use sp_core::H160;
use std::str::FromStr;

#[test]
fn trading_pair_works() {
	let see = CurrencyId::Token(TokenSymbol::SEE);
	let setr = CurrencyId::Token(TokenSymbol::SETR);
	let erc20 = CurrencyId::Erc20(EvmAddress::from_str("0x0000000000000000000000000000000000000000").unwrap());
	let see_setr_lp = CurrencyId::DexShare(DexShare::Token(TokenSymbol::SEE), DexShare::Token(TokenSymbol::SETR));
	let erc20_see_lp = CurrencyId::DexShare(
		DexShare::Token(TokenSymbol::SEE),
		DexShare::Erc20(EvmAddress::from_str("0x0000000000000000000000000000000000000000").unwrap()),
	);

	assert_eq!(
		TradingPair::from_currency_ids(setr, see).unwrap(),
		TradingPair(see, setr)
	);
	assert_eq!(
		TradingPair::from_currency_ids(see, setr).unwrap(),
		TradingPair(see, setr)
	);
	assert_eq!(
		TradingPair::from_currency_ids(erc20, see).unwrap(),
		TradingPair(see, erc20)
	);
	assert_eq!(TradingPair::from_currency_ids(see, see), None);

	assert_eq!(
		TradingPair::from_currency_ids(setr, see)
			.unwrap()
			.dex_share_currency_id(),
		see_setr_lp
	);
	assert_eq!(
		TradingPair::from_currency_ids(see, erc20)
			.unwrap()
			.dex_share_currency_id(),
		erc20_see_lp
	);
}

#[test]
fn currency_id_try_from_vec_u8_works() {
	assert_ok!(
		"SEE".as_bytes().to_vec().try_into(),
		CurrencyId::Token(TokenSymbol::SEE)
	);
}

#[test]
fn currency_id_into_u32_works() {
	let currency_id = DexShare::Token(TokenSymbol::SEE);
	assert_eq!(Into::<u32>::into(currency_id), 0x00);

	let currency_id = DexShare::Token(TokenSymbol::USSD);
	assert_eq!(Into::<u32>::into(currency_id), 0x01);

	let currency_id = DexShare::Erc20(EvmAddress::from_str("0x2000000000000000000000000000000000000000").unwrap());
	assert_eq!(Into::<u32>::into(currency_id), 0x20000000);

	let currency_id = DexShare::Erc20(EvmAddress::from_str("0x0000000000000001000000000000000000000000").unwrap());
	assert_eq!(Into::<u32>::into(currency_id), 0x01000000);

	let currency_id = DexShare::Erc20(EvmAddress::from_str("0x0000000000000000000000000000000000000001").unwrap());
	assert_eq!(Into::<u32>::into(currency_id), 0x01);

	let currency_id = DexShare::Erc20(EvmAddress::from_str("0x0000000000000000000000000000000000000000").unwrap());
	assert_eq!(Into::<u32>::into(currency_id), 0x00);
}

#[test]
fn currency_id_try_into_evm_address_works() {
	assert_eq!(
		EvmAddress::try_from(CurrencyId::Token(TokenSymbol::SEE,)),
		Ok(EvmAddress::from_str("0x0000000000000000000100000000000000000000").unwrap())
	);

	assert_eq!(
		EvmAddress::try_from(CurrencyId::DexShare(
			DexShare::Token(TokenSymbol::SEE),
			DexShare::Token(TokenSymbol::SETR),
		)),
		Ok(EvmAddress::from_str("0x0000000000000000000200000000000000000001").unwrap())
	);

	// No check the erc20 is mapped
	assert_eq!(
		EvmAddress::try_from(CurrencyId::DexShare(
			DexShare::Erc20(Default::default()),
			DexShare::Erc20(Default::default())
		)),
		Ok(EvmAddress::from_str("0x0000000000000000000201000000000100000000").unwrap())
	);

	let erc20 = EvmAddress::from_str("0x1111111111111111111111111111111111111111").unwrap();
	assert_eq!(EvmAddress::try_from(CurrencyId::Erc20(erc20)), Ok(erc20));

	assert_eq!(
		EvmAddress::try_from(CurrencyId::DexShare(
			DexShare::ForeignAsset(Default::default()),
			DexShare::ForeignAsset(Default::default())
		)),
		Ok(EvmAddress::from_str("0x0000000000000000000202000000000200000000").unwrap())
	);
}

#[test]
fn generate_function_selector_works() {
	#[module_evm_utility_macro::generate_function_selector]
	#[derive(RuntimeDebug, Eq, PartialEq)]
	#[repr(u32)]
	pub enum Action {
		Name = "name()",
		Symbol = "symbol()",
		Decimals = "decimals()",
		TotalSupply = "totalSupply()",
		BalanceOf = "balanceOf(address)",
		Transfer = "transfer(address,uint256)",
	}

	assert_eq!(Action::Name as u32, 0x06fdde03_u32);
	assert_eq!(Action::Symbol as u32, 0x95d89b41_u32);
	assert_eq!(Action::Decimals as u32, 0x313ce567_u32);
	assert_eq!(Action::TotalSupply as u32, 0x18160ddd_u32);
	assert_eq!(Action::BalanceOf as u32, 0x70a08231_u32);
	assert_eq!(Action::Transfer as u32, 0xa9059cbb_u32);
}

#[test]
fn is_system_contract_works() {
	assert!(is_system_contract(&H160::from_low_u64_be(0)));
	assert!(is_system_contract(&H160::from_low_u64_be(u64::max_value())));

	let mut bytes = [0u8; 20];
	bytes[SYSTEM_CONTRACT_ADDRESS_PREFIX.len() - 1] = 1u8;

	assert!(!is_system_contract(&bytes.into()));

	bytes = [0u8; 20];
	bytes[0] = 1u8;

	assert!(!is_system_contract(&bytes.into()));
}

#[test]
fn decode_gas_price_works() {
	const TX_FEE_PRE_GAS: u128 = 100_000_000_000u128; // 100 Gwei

	// tip = 0, gas_price = 0 Gwei, gas_limit = u64::MIN
	assert_eq!(decode_gas_price(u64::MIN, u64::MIN, TX_FEE_PRE_GAS), None);
	// tip = 0, gas_price = 99 Gwei, gas_limit = u64::MAX
	assert_eq!(decode_gas_price(99_999_999_999, u64::MIN, TX_FEE_PRE_GAS), None);
	// tip = 0, gas_price = 100 Gwei, gas_limit = u64::MIN
	assert_eq!(
		decode_gas_price(100_000_000_000, u64::MIN, TX_FEE_PRE_GAS),
		Some((0, 0))
	);
	// tip = 0, gas_price = 100 Gwei, gas_limit = u64::MAX
	assert_eq!(
		decode_gas_price(100_000_000_000, u64::MAX, TX_FEE_PRE_GAS),
		Some((0, 0))
	);

	// tip = 0, gas_price = 105 Gwei, gas_limit = u64::MIN
	assert_eq!(
		decode_gas_price(105_000_000_000, u64::MIN, TX_FEE_PRE_GAS),
		Some((0, u32::MAX))
	);
	// tip = 0, gas_price = 105 Gwei, gas_limit = u64::MAX
	assert_eq!(
		decode_gas_price(105_000_000_000, u64::MAX, TX_FEE_PRE_GAS),
		Some((0, u32::MAX))
	);

	// tip = 0, gas_price = u64::MAX, gas_limit = u64::MIN
	assert_eq!(
		decode_gas_price(u64::MAX, u64::MIN, TX_FEE_PRE_GAS),
		Some((0, 3_709_551_615))
	);
	// tip != 0, gas_price = u64::MAX, gas_limit = 1
	assert_eq!(decode_gas_price(u64::MAX, 1, TX_FEE_PRE_GAS), None);

	// tip != 200%, gas_price = 200 Gwei, gas_limit = 10000
	assert_eq!(
		decode_gas_price(200_000_000_000, 10_000, TX_FEE_PRE_GAS),
		Some((1_000_000_000, 0))
	);
}

#[test]
fn decode_gas_limit_works() {
	assert_eq!(decode_gas_limit(u64::MAX), (15_480_000, 32768));
	assert_eq!(decode_gas_limit(u64::MIN), (0, 0));
	assert_eq!(
		// u64::MAX = 4294967295
		decode_gas_limit(u64::MAX / 1000 * 1000 + 199),
		(15330000, 2u32.pow(MAX_GAS_LIMIT_CC))
	);
}
