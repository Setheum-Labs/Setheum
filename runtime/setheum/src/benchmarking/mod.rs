#![cfg(feature = "runtime-benchmarks")]

// module benchmarking
pub mod auction_manager;
pub mod cdp_engine;
pub mod cdp_treasury;
pub mod dex;
pub mod emergency_shutdown;
pub mod evm;
pub mod evm_accounts;
pub mod prices;
pub mod serp_treasury;
pub mod setmint;
pub mod transaction_payment;

// orml benchmarking
pub mod auction;
pub mod authority;
pub mod currencies;
pub mod oracle;
pub mod tokens;
pub mod utils;
