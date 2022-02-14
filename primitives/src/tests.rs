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

use super::*;
use crate::evm::{
	is_setheum_precompile, is_system_contract, EvmAddress, PRECOMPILE_ADDRESS_START, PREDEPLOY_ADDRESS_START,
	SYSTEM_CONTRACT_ADDRESS_PREFIX,
};
use frame_support::assert_ok;
use sp_core::H160;
use std::str::FromStr;

#[test]
fn trading_pair_works() {
	let setm = CurrencyId::Token(TokenSymbol::SETM);
	let setusd = CurrencyId::Token(TokenSymbol::SETUSD);
	let erc20 = CurrencyId::Erc20(EvmAddress::from_str("0x0000000000000000000000000000000000000000").unwrap());
	let setm_setusd_lp = CurrencyId::DexShare(DexShare::Token(TokenSymbol::SETM), DexShare::Token(TokenSymbol::SETUSD));
	let erc20_setm_lp = CurrencyId::DexShare(
		DexShare::Token(TokenSymbol::SETM),
		DexShare::Erc20(EvmAddress::from_str("0x0000000000000000000000000000000000000000").unwrap()),
	);

	assert_eq!(
		TradingPair::from_currency_ids(setusd, setm).unwrap(),
		TradingPair(setm, setusd)
	);
	assert_eq!(
		TradingPair::from_currency_ids(setm, setusd).unwrap(),
		TradingPair(setm, setusd)
	);
	assert_eq!(
		TradingPair::from_currency_ids(erc20, setm).unwrap(),
		TradingPair(setm, erc20)
	);
	assert_eq!(TradingPair::from_currency_ids(setm, setm), None);

	assert_eq!(
		TradingPair::from_currency_ids(setusd, setm)
			.unwrap()
			.dex_share_currency_id(),
		setm_setusd_lp
	);
	assert_eq!(
		TradingPair::from_currency_ids(setm, erc20)
			.unwrap()
			.dex_share_currency_id(),
		erc20_setm_lp
	);
}

#[test]
fn currency_id_try_from_vec_u8_works() {
	assert_ok!(
		"SETM".as_bytes().to_vec().try_into(),
		CurrencyId::Token(TokenSymbol::SETM)
	);
}

#[test]
fn currency_id_into_u32_works() {
	let currency_id = DexShare::Token(TokenSymbol::SETM);
	assert_eq!(Into::<u32>::into(currency_id), 0x00);

	let currency_id = DexShare::Token(TokenSymbol::SETUSD);
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
		EvmAddress::try_from(CurrencyId::Token(TokenSymbol::SETM,)),
		Ok(EvmAddress::from_str("0x0000000000000000000100000000000000000000").unwrap())
	);

	assert_eq!(
		EvmAddress::try_from(CurrencyId::DexShare(
			DexShare::Token(TokenSymbol::SETM),
			DexShare::Token(TokenSymbol::SETUSD),
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
}

#[test]
fn generate_function_selector_works() {
	#[module_evm_utiltity_macro::generate_function_selector]
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
	assert!(is_system_contract(H160::from_low_u64_be(0)));
	assert!(is_system_contract(H160::from_low_u64_be(u64::max_value())));

	let mut bytes = [0u8; 20];
	bytes[SYSTEM_CONTRACT_ADDRESS_PREFIX.len() - 1] = 1u8;

	assert!(!is_system_contract(bytes.into()));

	bytes = [0u8; 20];
	bytes[0] = 1u8;

	assert!(!is_system_contract(bytes.into()));
}

#[test]
fn is_setheum_precompile_works() {
	assert!(!is_setheum_precompile(H160::from_low_u64_be(0)));
	assert!(!is_setheum_precompile(H160::from_low_u64_be(
		PRECOMPILE_ADDRESS_START.to_low_u64_be() - 1
	)));
	assert!(is_setheum_precompile(PRECOMPILE_ADDRESS_START));
	assert!(is_setheum_precompile(H160::from_low_u64_be(
		PREDEPLOY_ADDRESS_START.to_low_u64_be() - 1
	)));
	assert!(!is_setheum_precompile(PREDEPLOY_ADDRESS_START));
	assert!(!is_setheum_precompile([1u8; 20].into()));
}
