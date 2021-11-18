
use crate::{
	SetheumOracle, AccountId, Balance, Currencies,
	CurrencyId, MinimumCount, OperatorMembershipSetheum,
	Price, Runtime, TokenSymbol,
};

use frame_benchmarking::account;
use frame_support::traits::tokens::fungibles;
use frame_support::{assert_ok, traits::Contains};
use frame_system::RawOrigin;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use sp_runtime::{
	traits::{SaturatedConversion, StaticLookup},
	DispatchResult,
};
use sp_std::prelude::*;

pub fn lookup_of_account(who: AccountId) -> <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source {
	<Runtime as frame_system::Config>::Lookup::unlookup(who)
}

pub fn set_balance(currency_id: CurrencyId, who: &AccountId, balance: Balance) {
	let _ = <Currencies as MultiCurrencyExtended<_>>::update_balance(currency_id, &who, balance.saturated_into());
	assert_eq!(
		<Currencies as MultiCurrency<_>>::free_balance(currency_id, who),
		balance
	);
}

pub fn set_setheum_balance(who: &AccountId, balance: Balance) {
	set_balance(CurrencyId::Token(TokenSymbol::SETM), who, balance)
}

pub fn feed_price(prices: Vec<(CurrencyId, Price)>) -> DispatchResult {
	for i in 0..MinimumCount::get() {
		let oracle: AccountId = account("oracle", 0, i);
		if !OperatorMembershipSetheum::contains(&oracle) {
			OperatorMembershipSetheum::add_member(RawOrigin::Root.into(), oracle.clone())?;
		}
		SetheumOracle::feed_values(RawOrigin::Signed(oracle).into(), prices.to_vec())
			.map_or_else(|e| Err(e.error), |_| Ok(()))?;
	}

	Ok(())
}

pub fn set_setheum_balance(who: &AccountId, balance: Balance) {
	set_balance(CurrencyId::Token(TokenSymbol::SETM), who, balance)
}

pub fn set_balance_fungibles(currency_id: CurrencyId, who: &AccountId, balance: Balance) {
	assert_ok!(<orml_tokens::Pallet<Runtime> as fungibles::Mutate<AccountId>>::mint_into(currency_id, who, balance));
}

#[cfg(test)]
pub mod tests {
	pub fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default()
			.build_storage::<crate::Runtime>()
			.unwrap()
			.into()
	}
}
