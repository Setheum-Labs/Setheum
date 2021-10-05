
use super::*;
use crate::evm::EvmAddress;
use frame_support::assert_ok;
use std::{
	convert::{TryFrom, TryInto},
	str::FromStr,
};

#[test]
fn trading_pair_works() {
	let dnar = CurrencyId::Token(TokenSymbol::DNAR);
	let setusd = CurrencyId::Token(TokenSymbol::SETUSD);
	let erc20 = CurrencyId::Erc20(EvmAddress::from_str("0x0000000000000000000000000000000000000000").unwrap());
	let dnar_setusd_lp = CurrencyId::DexShare(DexShare::Token(TokenSymbol::DNAR), DexShare::Token(TokenSymbol::SETUSD));
	let erc20_dnar_lp = CurrencyId::DexShare(
		DexShare::Token(TokenSymbol::DNAR),
		DexShare::Erc20(EvmAddress::from_str("0x0000000000000000000000000000000000000000").unwrap()),
	);

	assert_eq!(
		TradingPair::from_currency_ids(setusd, dnar).unwrap(),
		TradingPair(dnar, setusd)
	);
	assert_eq!(
		TradingPair::from_currency_ids(dnar, setusd).unwrap(),
		TradingPair(dnar, setusd)
	);
	assert_eq!(
		TradingPair::from_currency_ids(erc20, dnar).unwrap(),
		TradingPair(dnar, erc20)
	);
	assert_eq!(TradingPair::from_currency_ids(dnar, dnar), None);

	assert_eq!(
		TradingPair::from_currency_ids(setusd, dnar)
			.unwrap()
			.dex_share_currency_id(),
		dnar_setusd_lp
	);
	assert_eq!(
		TradingPair::from_currency_ids(dnar, erc20)
			.unwrap()
			.dex_share_currency_id(),
		erc20_dnar_lp
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
	assert_eq!(Into::<u32>::into(currency_id), 0x04);

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
		Ok(EvmAddress::from_str("0x0000000000000000000000000000000001000000").unwrap())
	);

	assert_eq!(
		EvmAddress::try_from(CurrencyId::DexShare(
			DexShare::Token(TokenSymbol::SETM),
			DexShare::Token(TokenSymbol::SETUSD),
		)),
		Ok(EvmAddress::from_str("0x0000000000000000000000010000000000000004").unwrap())
	);

	assert_eq!(
		EvmAddress::try_from(CurrencyId::DexShare(
			DexShare::Erc20(Default::default()),
			DexShare::Erc20(Default::default())
		)),
		Err(())
	);

	let erc20 = EvmAddress::from_str("0x1111111111111111111111111111111111111111").unwrap();
	assert_eq!(EvmAddress::try_from(CurrencyId::Erc20(erc20)), Ok(erc20));
}

#[test]
fn generate_function_selector_works() {
	#[primitives_proc_macro::generate_function_selector]
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
