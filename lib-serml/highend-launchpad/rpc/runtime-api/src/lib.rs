//! Runtime API definition for launchpad-crowdsales module.

#![cfg_attr(not(feature = "std"), no_std)]
// The `too_many_arguments` warning originates from `decl_runtime_apis` macro.
#![allow(clippy::too_many_arguments)]
// The `unnecessary_mut_passed` warning originates from `decl_runtime_apis` macro.
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_runtime::traits::MaybeDisplay;
use sp_std::prelude::*;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait LaunchpadCrowdsalesApi<Balance, BlockNumber, CurrencyId> where
		Balance: Codec + MaybeDisplay,
		BlockNumber: Codec + MaybeDisplay,
		CurrencyId: Codec + MaybeDisplay,
	{
		// Get the total amount of funds raised in the entire protocol.
		fn get_total_amounts_raised() -> Vec<(CurrencyId, Balance)>;
	}
}
