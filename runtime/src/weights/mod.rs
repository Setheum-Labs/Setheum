//! A list of the different weight modules for our runtime.
#![allow(clippy::unnecessary_cast)]

pub mod auction_manager;
pub mod cdp_engine;
pub mod cdp_treasury;
pub mod module_currencies;
pub mod module_dex;
pub mod emergency_shutdown;
pub mod module_evm;
pub mod module_evm_accounts;
pub mod serp_setmint;
pub mod module_nft;
pub mod module_prices;
pub mod module_transaction_pause;
pub mod module_transaction_payment;

pub mod orml_auction;
pub mod orml_authority;
pub mod orml_oracle;
pub mod orml_tokens;
