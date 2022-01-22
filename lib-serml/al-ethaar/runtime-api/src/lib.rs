#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_runtime::traits::MaybeDisplay;
use sp_std::prelude::*;

use pallet_open_grant::{Project};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait OpenGrantApi<AccountId, BlockNumber> where AccountId: Codec + MaybeDisplay,  BlockNumber: Codec + MaybeDisplay {
		fn get_projects() -> Vec<Project<AccountId, BlockNumber>>;
	}
}
