#![cfg(feature = "runtime-benchmarks")]

use sp_runtime::traits::AccountIdConversion;
pub mod utils;

// module benchmarking
pub mod auction_manager;
pub mod cdp_engine;
pub mod cdp_treasury;
pub mod currencies;
pub mod dex;
pub mod emergency_shutdown;
pub mod evm;
pub mod evm_accounts;
pub mod serp_setmint;
pub mod prices;
pub mod transaction_pause;
pub mod transaction_payment;

// orml benchmarking
pub mod auction;
pub mod authority;
pub mod oracle;
pub mod tokens;
