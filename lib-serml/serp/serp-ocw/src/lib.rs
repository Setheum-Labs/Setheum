// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

#[cfg(test)]
mod tests;

// We have to import a few things
use rstd::{prelude::*};
use primitives::crypto::KeyTypeId;
use support::{decl_module, decl_storage, decl_event, dispatch,
  debug, storage::IterableStorageMap, traits::Get};
use system::offchain::{SubmitTransaction};
use system::{ensure_none};
use simple_json::{self, json::JsonValue};
use runtime_io::{self, misc::print_utf8 as print_bytes};
#[cfg(not(feature = "std"))]
use num_traits::float::FloatCore;
use codec::Encode;
use sp_runtime::{
  offchain::http,
  transaction_validity::{
    TransactionValidity, TransactionLongevity, ValidTransaction, InvalidTransaction
  }
};

type Result<T> = core::result::Result<T, &'static str>;

/// Our local KeyType.
///
/// For security reasons the offchain worker doesn't have direct access to the keys
/// but only to app-specific subkeys, which are defined and grouped by their `KeyTypeId`.
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ofpf");

// REVIEW-CHECK: is it necessary to wrap-around storage vector at `MAX_VEC_LEN`?
pub const MAX_VEC_LEN: usize = 1000;

pub mod crypto {
  pub use super::KEY_TYPE;
  use sp_runtime::app_crypto::{app_crypto, sr25519};
  app_crypto!(sr25519, KEY_TYPE);
}

// Update the feeds with SetCurrencies when they are listed -
//
// // TODO: update - listed SetCurrency!
//
// - we currently use other currency feed prices to satisfy the SetCurrency feeds.
pub const FETCHED_CRYPTOS: [(&[u8], &[u8], &[u8]); 6] = [
  // CRYPTO PRICES => USD
  // feed as is to oracle.
  //
  // bitstamp feeds (crypto and fiat)
  (b"DNAR", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/btcusd"),
  (b"DRAM", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/ethusd"),
  (b"BTC", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/btcusd"),
  (b"ETH", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/ethusd"),
  (b"SETR", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/usdtusd"),
  (b"SETUSD", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/usdtusd"),
  (b"SETEUR", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/eurtusd"),
  (b"EUR", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/eurusd"),
  (b"GBP", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/gbpusd"),
  // (b"SETGBP", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/setgbpusd"),
  // (b"SETCHF", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/setchfusd"),
  // (b"SETSAR", b"bitstamp", b"https://www.bitstamp.net/api/v2/ticker/setsarusd"),
  //
  // coinbase feeds (crypto only)
  (b"DNAR", b"coinbase", b"https://api.pro.coinbase.com/products/BTC-USD/ticker"),
  (b"DRAM", b"coinbase", b"https://api.pro.coinbase.com/products/ETH-USD/ticker"),
  (b"BTC", b"coinbase", b"https://api.pro.coinbase.com/products/BTC-USD/ticker"),
  (b"ETH", b"coinbase", b"https://api.pro.coinbase.com/products/ETH-USD/ticker"),
  (b"SETR", b"coinbase", b"https://api.pro.coinbase.com/products/USDT-USD/ticker"),
  (b"SETUSD", b"coinbase", b"https://api.pro.coinbase.com/products/USDT-USD/ticker"),
  // (b"SETEUR", b"coinbase", b"https://min-api.cryptocompare.com/data/price?fsym=SETEUR&tsyms=USD"),
  // (b"SETGBP", b"coinbase", b"https://min-api.cryptocompare.com/data/price?fsym=SETGBP&tsyms=USDC"),
  // (b"SETCHF", b"coinbase", b"https://min-api.cryptocompare.com/data/price?fsym=SETCHF&tsyms=USD"),
  // (b"SETSAR", b"coinbase", b"https://min-api.cryptocompare.com/data/price?fsym=SETSAR&tsyms=USD"),
  //
  // coincap feeds (crypto only)
  (b"DNAR", b"coincap", b"https://api.coincap.io/v2/assets/bitcoin"),
  (b"DRAM", b"coincap", b"https://api.coincap.io/v2/assets/ethereum"),
  (b"BTC", b"coincap", b"https://api.coincap.io/v2/assets/bitcoin"),
  (b"ETH", b"coincap", b"https://api.coincap.io/v2/assets/ethereum"),
  (b"SETR", b"coincap", b"https://api.coincap.io/v2/assets/stasis-euro"),
  (b"SETUSD", b"coincap", b"https://api.coincap.io/v2/assets/tether"),
  (b"SETEUR", b"coincap", b"https://api.coincap.io/v2/assets/stasis-euro"),
  // (b"SETGBP", b"coincap", b"https://api.coincap.io/v2/assets/setpound"),
  // (b"SETCHF", b"coincap", b"https://api.coincap.io/v2/assets/setfranc"),
  // (b"SETSAR", b"coincap", b"https://api.coincap.io/v2/assets/setriyal"),
  //
  // coingecko feeds (crypto only)
  (b"DNAR", b"coingecko", b"https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd"),
  (b"DRAM", b"coingecko", b"https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd"),
  (b"BTC", b"coingecko", b"https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd"),
  (b"ETH", b"coingecko", b"https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd"),
  (b"SETR", b"coingecko", b"https://api.coingecko.com/api/v3/simple/price?ids=cryptofranc&vs_currencies=usd"),
  (b"SETUSD", b"coingecko", b"https://api.coingecko.com/api/v3/simple/price?ids=busd&vs_currencies=usd"),
  (b"SETEUR", b"coingecko", b"https://api.coingecko.com/api/v3/simple/price?ids=upper-euro&vs_currencies=usd"),
  (b"SETGBP", b"coingecko", b"https://api.coingecko.com/api/v3/simple/price?ids=upper-pound&vs_currencies=usd"),
  (b"SETCHF", b"coingecko", b"https://api.coingecko.com/api/v3/simple/price?ids=cryptofranc&vs_currencies=usd"),
  // (b"SETSAR", b"coingecko", b"https://min-api.cryptocompare.com/data/price?fsym=SETSAR&tsyms=USD"),
  //
  // coinpaprika feeds (crypto only)
  (b"DNAR", b"coinpaprika", b"https://api.coinpaprika.com/v1/tickers/btc-bitcoin"),
  (b"DRAM", b"coinpaprika", b"https://api.coinpaprika.com/v1/tickers/eth-ethereum"),
  (b"BTC", b"coinpaprika", b"https://api.coinpaprika.com/v1/tickers/btc-bitcoin"),
  (b"ETH", b"coinpaprika", b"https://api.coinpaprika.com/v1/tickers/eth-ethereum"),
  (b"SETR", b"coinpaprika", b"https://api.coinpaprika.com/v1/tickers/gbp-pound-sterling-token"),
  (b"SETUSD", b"coinpaprika", b"https://api.coinpaprika.com/v1/tickers/usdt-tether"),
  (b"SETEUR", b"coinpaprika", b"https://api.coinpaprika.com/v1/tickers/eurs-stasis-eurs"),
  (b"SETGBP", b"coinpaprika", b"https://api.coinpaprika.com/v1/tickers/gbp-pound-sterling-token"),
  (b"SETCHF", b"coinpaprika", b"https://api.coinpaprika.com/v1/tickers/chf-swiss-franc-token"),
  // (b"SETSAR", b"coinpaprika", b"https://api.coinpaprika.com/v1/tickers/setsar-setriyal"),
  //
  // cryptocompare feeds (crypto only)
  (b"DNAR", b"cryptocompare", b"https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD"),
  (b"DRAM", b"cryptocompare", b"https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=USD"),
  (b"BTC", b"cryptocompare", b"https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD"),
  (b"ETH", b"cryptocompare", b"https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=USD"),
  (b"SETR", b"cryptocompare", b"https://min-api.cryptocompare.com/data/price?fsym=EURS&tsyms=USD"),
  (b"SETUSD", b"cryptocompare", b"https://min-api.cryptocompare.com/data/price?fsym=USDT&tsyms=USD"),
  (b"SETEUR", b"cryptocompare", b"https://min-api.cryptocompare.com/data/price?fsym=EURS&tsyms=USD"),
  (b"SETGBP", b"cryptocompare", b"https://min-api.cryptocompare.com/data/price?fsym=BGBP&tsyms=USDC"),
  (b"SETCHF", b"cryptocompare", b"https://min-api.cryptocompare.com/data/price?fsym=XCHF&tsyms=USD"),
  (b"SETSAR", b"cryptocompare", b"https://min-api.cryptocompare.com/data/price?fsym=POLNX&tsyms=USD"),
  //
  // cryptowatftx feeds (crypto only)
  (b"DNAR", b"cryptowatftx", b"https://api.cryptowat.ch/markets/ftx/btcusd/price"),
  (b"DRAM", b"cryptowatftx", b"https://api.cryptowat.ch/markets/ftx/ethusd/price"),
  (b"BTC", b"cryptowatftx", b"https://api.cryptowat.ch/markets/ftx/btcusd/price"),
  (b"ETH", b"cryptowatftx", b"https://api.cryptowat.ch/markets/ftx/ethusd/price"),
  (b"SETR", b"cryptowatftx", b"https://api.cryptowat.ch/markets/ftx/usdtusd/price"),
  (b"SETUSD", b"cryptowatftx", b"https://api.cryptowat.ch/markets/ftx/usdtusd/price"),
  //
  // exchangerate.host feeds (fiat only)
  (b"USD", b"exchangeratehost", b"https://api.exchangerate.host/convert?from=USD&to=USD"),
  (b"EUR", b"exchangeratehost", b"https://api.exchangerate.host/convert?from=EUR&to=USD"),
  (b"GBP", b"exchangeratehost", b"https://api.exchangerate.host/convert?from=GBP&to=USD"),
  (b"CHF", b"exchangeratehost", b"https://api.exchangerate.host/convert?from=CHF&to=USD"),
  (b"SAR", b"exchangeratehost", b"https://api.exchangerate.host/convert?from=SAR&to=USD"),
  //
  // gemini feeds (crypto only)
  (b"DNAR", b"gemini", b"https://api.gemini.com/v1/pricefeed/btcusd"),
  (b"DRAM", b"gemini", b"https://api.gemini.com/v1/pricefeed/ethusd"),
  (b"BTC", b"gemini", b"https://api.gemini.com/v1/pricefeed/btcusd"),
  (b"ETH", b"gemini", b"https://api.gemini.com/v1/pricefeed/ethusd"),
];

pub trait FetchPriceFor {
  fn get_price_for(symbol: &[u8]) -> u64;
}

impl<T: Trait> FetchPriceFor for Module<T> {
  fn get_price_for(symbol: &[u8]) -> u64 {
    let (_, price) = <TokenAggPPMap<T>>::get(symbol);
    // if price == Default::default() {
    //   return None;
    // }
    price.into()
  }
}

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait {
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
  type Call: From<Call<Self>>;
  // Wait period between automated fetches. Set to 0 disable this feature.
  //   Then you need to manucally kickoff pricefetch
  type BlockFetchDur: Get<Self::BlockNumber>;
}

type StrVecBytes = Vec<u8>;
// Q: Is it possible to do:
// type PricePoint = (Moment, u64);
//
// So inside Event enum, I can do:
// ```
// pub enum Event<T> where Moment = <T as timestamp::Trait>::Moment {
//   FetchedPrice(StrVecBytes, StrVecBytes, PricePoint),
//   ...

decl_event!(
  pub enum Event<T> where Moment = <T as timestamp::Trait>::Moment {
    FetchedPrice(StrVecBytes, StrVecBytes, Moment, u64),
    AggregatedPrice(StrVecBytes, Moment, u64),
  }
);

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as PriceFetch {
    // mapping of token sym -> (timestamp, price)
    //   price has been inflated by 10,000, and in USD.
    //   When used, it should be divided by 10,000.
    // Using linked map for easy traversal from offchain worker or UI
    TokenSrcPPMap: map hasher(blake2_128_concat) StrVecBytes => Vec<(T::Moment, u64)>;

    // storage about aggregated price points (calculated with our logic)
    TokenAggPPMap: map hasher(blake2_128_concat) StrVecBytes => (T::Moment, u64);
  }
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your module
    fn deposit_event() = default;

    pub fn record_price(
      origin,
      _block: T::BlockNumber,
      crypto_info: (StrVecBytes, StrVecBytes, StrVecBytes),
      price: u64
    ) -> dispatch::DispatchResult {
      // Ensuring this is an unsigned tx
      ensure_none(origin)?;

      let (sym, remote_src) = (crypto_info.0, crypto_info.1);
      let now = <timestamp::Module<T>>::get();

      // Debug printout
      debug::info!("record_price: {:?}, {:?}, {:?}",
        core::str::from_utf8(&sym).map_err(|_| "`sym` conversion error")?,
        core::str::from_utf8(&remote_src).map_err(|_| "`remote_src` conversion error")?,
        price
      );

      <TokenSrcPPMap<T>>::mutate(&sym, |pp_vec| pp_vec.push((now, price)));

      // Spit out an event and Add to storage
      Self::deposit_event(RawEvent::FetchedPrice(sym, remote_src, now, price));

      Ok(())
    }

    pub fn record_agg_pp(
      origin,
      _block: T::BlockNumber,
      sym: StrVecBytes,
      price: u64
    ) -> dispatch::DispatchResult {
      // Debug printout
      debug::info!("record_agg_pp: {}: {:?}",
        core::str::from_utf8(&sym).map_err(|_| "`sym` string conversion error")?,
        price
      );

      // Ensuring this is an unsigned tx
      ensure_none(origin)?;

      let now = <timestamp::Module<T>>::get();

      // Record in the storage
      let price_pt = (now.clone(), price.clone());
      <TokenAggPPMap<T>>::insert(&sym, price_pt);

      // Remove relevant storage items
      <TokenSrcPPMap<T>>::remove(&sym);

      // Spit the event
      Self::deposit_event(RawEvent::AggregatedPrice(
        sym.clone(), now.clone(), price.clone()));

      Ok(())
    }

    fn offchain_worker(block: T::BlockNumber) {
      let duration = T::BlockFetchDur::get();

      // Type I task: fetch price
      if duration > 0.into() && block % duration == 0.into() {
        for (sym, remote_src, remote_url) in FETCHED_CRYPTOS.iter() {
          if let Err(e) = Self::fetch_price(block, *sym, *remote_src, *remote_url) {
            debug::error!("Error fetching: {:?}, {:?}: {:?}",
              core::str::from_utf8(sym).unwrap(),
              core::str::from_utf8(remote_src).unwrap(),
              e);
          }
        }
      }

      // Type II task: aggregate price
      TokenSrcPPMap::<T>::iter()
        // filter those to be updated
        .filter(|(_, vec)| vec.len() > 0)
        .for_each(|(sym, _)| {
          if let Err(e) = Self::aggregate_pp(block, &sym) {
            debug::error!("Error aggregating price of {:?}: {:?}",
              core::str::from_utf8(&sym).unwrap(), e);
          }
        });
    } // end of `fn offchain_worker()`
  }
}

impl<T: Trait> Module<T> {
  fn fetch_json<'a>(remote_url: &'a [u8]) -> Result<JsonValue> {
    let remote_url_str = core::str::from_utf8(remote_url)
      .map_err(|_| "Error in converting remote_url to string")?;

    let pending = http::Request::get(remote_url_str).send()
      .map_err(|_| "Error in sending http GET request")?;

    let response = pending.wait()
      .map_err(|_| "Error in waiting http response back")?;

    if response.code != 200 {
      debug::warn!("Unexpected status code: {}", response.code);
      return Err("Non-200 status code returned from http request");
    }

    let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();

    // Print out the whole JSON blob
    print_bytes(&json_result);

    let json_val: JsonValue = simple_json::parse_json(
      &core::str::from_utf8(&json_result).map_err(|_| "JSON result cannot convert to string")?)
      .map_err(|_| "JSON parsing error")?;

    Ok(json_val)
  }

  fn fetch_price<'a>(
    block: T::BlockNumber,
    sym: &'a [u8],
    remote_src: &'a [u8],
    remote_url: &'a [u8]
  ) -> Result<()> {
    debug::info!("fetch price: {:?}:{:?}",
      core::str::from_utf8(sym).unwrap(),
      core::str::from_utf8(remote_src).unwrap()
    );

    let json = Self::fetch_json(remote_url)?;

    let price = match remote_src {
      src if src == b"coincap" => Self::fetch_price_from_bitstamp(json)
        .map_err(|_| "fetch_price_from_bitstamp error"),
      src if src == b"coincap" => Self::fetch_price_from_coinbase(json)
        .map_err(|_| "fetch_price_from_coinbase error"),
      src if src == b"coincap" => Self::fetch_price_from_coincap(json)
        .map_err(|_| "fetch_price_from_coincap error"),
      src if src == b"coincap" => Self::fetch_price_from_coingecko(json)
        .map_err(|_| "fetch_price_from_coingecko error"),
      src if src == b"coincap" => Self::fetch_price_from_coinpaprika(json)
        .map_err(|_| "fetch_price_from_coinpaprika error"),
      src if src == b"coincap" => Self::fetch_price_from_cryptocompare(json)
        .map_err(|_| "fetch_price_from_cryptocompare error"),
      src if src == b"coincap" => Self::fetch_price_from_cryptowatftx(json)
        .map_err(|_| "fetch_price_from_cryptowatftx error"),
      src if src == b"coincap" => Self::fetch_price_from_exchangeratehost(json)
        .map_err(|_| "fetch_price_from_exchangeratehost error"),
      src if src == b"coincap" => Self::fetch_price_from_gemini(json)
        .map_err(|_| "fetch_price_from_gemini error"),
      _ => Err("Unknown remote source"),
    }?;

    let call = Call::record_price(
      block,
      (sym.to_vec(), remote_src.to_vec(), remote_url.to_vec()),
      price
    );

    // Unsigned tx
    T::SubmitTransaction::submit_unsigned_transaction(call.into())
      .map_err(|_| "fetch_price: submit_unsigned(call) error")

    // Signed tx
    // let local_accts = T::SubmitTransaction::find_local_keys(None);
    // let (local_acct, local_key) = local_accts[0];
    // debug::info!("acct: {:?}", local_acct);
    // T::SignAndSubmitTransaction::sign_and_submit(call, local_key);

    // T::SubmitSignedTransaction::submit_signed(call);
  }

  fn vecchars_to_vecbytes<I: IntoIterator<Item = char> + Clone>(it: &I) -> Vec<u8> {
    it.clone().into_iter().map(|c| c as u8).collect::<_>()
  }

  fn fetch_price_from_bitstamp(json_val: JsonValue) -> Result<u64> {
    // Expected JSON shape:
    //   r#"{"data":{"last":"8172.2628346190447316"}}"#;

    const PRICE_KEY: &[u8] = b"last";
    let data = json_val.get_object()[0].1.get_object();

    let (_, v) = data.iter()
      .filter(|(k, _)| PRICE_KEY.to_vec() == Self::vecchars_to_vecbytes(k))
      .nth(0)
      .ok_or("fetch_price_from_bitstamp: JSON does not conform to expectation")?;

    // `val` contains the price, such as "222.333" in bytes form
    let val_u8: Vec<u8> = v.get_bytes();

    // Convert to number
    let val_f64: f64 = core::str::from_utf8(&val_u8)
      .map_err(|_| "fetch_price_from_bitstamp: val_f64 convert to string error")?
      .parse::<f64>()
      .map_err(|_| "fetch_price_from_bitstamp: val_u8 parsing to f64 error")?;
    Ok((val_f64 * 10000.).round() as u64)
  }

  fn fetch_price_from_coinbase(json_val: JsonValue) -> Result<u64> {
    // Expected JSON shape:
    //   r#"{"data":{"price":"8172.2628346190447316"}}"#;

    const PRICE_KEY: &[u8] = b"price";
    let data = json_val.get_object()[0].1.get_object();

    let (_, v) = data.iter()
      .filter(|(k, _)| PRICE_KEY.to_vec() == Self::vecchars_to_vecbytes(k))
      .nth(0)
      .ok_or("fetch_price_from_coinbase: JSON does not conform to expectation")?;

    // `val` contains the price, such as "222.333" in bytes form
    let val_u8: Vec<u8> = v.get_bytes();

    // Convert to number
    let val_f64: f64 = core::str::from_utf8(&val_u8)
      .map_err(|_| "fetch_price_from_coinbase: val_f64 convert to string error")?
      .parse::<f64>()
      .map_err(|_| "fetch_price_from_coinbase: val_u8 parsing to f64 error")?;
    Ok((val_f64 * 10000.).round() as u64)
  }

  fn fetch_price_from_coincap(json_val: JsonValue) -> Result<u64> {
    // Expected JSON shape:
    //   r#"{"data":{"priceUsd":"8172.2628346190447316"}}"#;

    const PRICE_KEY: &[u8] = b"priceUsd";
    let data = json_val.get_object()[0].1.get_object();

    let (_, v) = data.iter()
      .filter(|(k, _)| PRICE_KEY.to_vec() == Self::vecchars_to_vecbytes(k))
      .nth(0)
      .ok_or("fetch_price_from_coincap: JSON does not conform to expectation")?;

    // `val` contains the price, such as "222.333" in bytes form
    let val_u8: Vec<u8> = v.get_bytes();

    // Convert to number
    let val_f64: f64 = core::str::from_utf8(&val_u8)
      .map_err(|_| "fetch_price_from_coincap: val_f64 convert to string error")?
      .parse::<f64>()
      .map_err(|_| "fetch_price_from_coincap: val_u8 parsing to f64 error")?;
    Ok((val_f64 * 10000.).round() as u64)
  }

  fn fetch_price_from_coingecko(json_val: JsonValue) -> Result<u64> {
    // Expected JSON shape:
    //   r#"{"data":{"usd":"8172.2628346190447316"}}"#;

    const PRICE_KEY: &[u8] = b"usd";
    let data = json_val.get_object()[0].1.get_object();

    let (_, v) = data.iter()
      .filter(|(k, _)| PRICE_KEY.to_vec() == Self::vecchars_to_vecbytes(k))
      .nth(0)
      .ok_or("fetch_price_from_coingecko: JSON does not conform to expectation")?;

    // `val` contains the price, such as "222.333" in bytes form
    let val_u8: Vec<u8> = v.get_bytes();

    // Convert to number
    let val_f64: f64 = core::str::from_utf8(&val_u8)
      .map_err(|_| "fetch_price_from_coingecko: val_f64 convert to string error")?
      .parse::<f64>()
      .map_err(|_| "fetch_price_from_coingecko: val_u8 parsing to f64 error")?;
    Ok((val_f64 * 10000.).round() as u64)
  }

  fn fetch_price_from_coinpaprika(json_val: JsonValue) -> Result<u64> {
    // Expected JSON shape:
    //   r#"{"data":{"price":"8172.2628346190447316"}}"#;

    const PRICE_KEY: &[u8] = b"price";
    let data = json_val.get_object()[0].1.get_object();

    let (_, v) = data.iter()
      .filter(|(k, _)| PRICE_KEY.to_vec() == Self::vecchars_to_vecbytes(k))
      .nth(0)
      .ok_or("fetch_price_from_coinpaprika: JSON does not conform to expectation")?;

    // `val` contains the price, such as "222.333" in bytes form
    let val_u8: Vec<u8> = v.get_bytes();

    // Convert to number
    let val_f64: f64 = core::str::from_utf8(&val_u8)
      .map_err(|_| "fetch_price_from_coinpaprika: val_f64 convert to string error")?
      .parse::<f64>()
      .map_err(|_| "fetch_price_from_coinpaprika: val_u8 parsing to f64 error")?;
    Ok((val_f64 * 10000.).round() as u64)
  }

  fn fetch_price_from_cryptocompare(json_val: JsonValue) -> Result<u64> {
    // Expected JSON shape:
    //   r#"{"USD": 7064.16}"#;
    let val_f64: f64 = json_val.get_object()[0].1.get_number_f64();
    Ok((val_f64 * 10000.).round() as u64)
  }

  fn fetch_price_from_cryptowatftx(json_val: JsonValue) -> Result<u64> {
    // Expected JSON shape:
    //   r#"{"result":{"price":"8172.2628346190447316"}}"#;

    const PRICE_KEY: &[u8] = b"price";
    let result = json_val.get_object()[0].1.get_object();

    let (_, v) = result.iter()
      .filter(|(k, _)| PRICE_KEY.to_vec() == Self::vecchars_to_vecbytes(k))
      .nth(0)
      .ok_or("fetch_price_from_cryptowatftx: JSON does not conform to expectation")?;

    // `val` contains the price, such as "222.333" in bytes form
    let val_u8: Vec<u8> = v.get_bytes();

    // Convert to number
    let val_f64: f64 = core::str::from_utf8(&val_u8)
      .map_err(|_| "fetch_price_from_cryptowatftx: val_f64 convert to string error")?
      .parse::<f64>()
      .map_err(|_| "fetch_price_from_cryptowatftx: val_u8 parsing to f64 error")?;
    Ok((val_f64 * 10000.).round() as u64)
  }

  fn fetch_price_from_exchangeratehost(json_val: JsonValue) -> Result<u64> {
    // Expected JSON shape:
    //   r#"{"data":{"result":"8172.2628346190447316"}}"#;

    const PRICE_KEY: &[u8] = b"result";
    let data = json_val.get_object()[0].1.get_object();

    let (_, v) = data.iter()
      .filter(|(k, _)| PRICE_KEY.to_vec() == Self::vecchars_to_vecbytes(k))
      .nth(0)
      .ok_or("fetch_price_from_exchangeratehost: JSON does not conform to expectation")?;

    // `val` contains the price, such as "222.333" in bytes form
    let val_u8: Vec<u8> = v.get_bytes();

    // Convert to number
    let val_f64: f64 = core::str::from_utf8(&val_u8)
      .map_err(|_| "fetch_price_from_exchangeratehost: val_f64 convert to string error")?
      .parse::<f64>()
      .map_err(|_| "fetch_price_from_exchangeratehost: val_u8 parsing to f64 error")?;
    Ok((val_f64 * 10000.).round() as u64)
  }

  fn fetch_price_from_gemini(json_val: JsonValue) -> Result<u64> {
    // Expected JSON shape:
    //   r#"{"data":{"price":"8172.2628346190447316"}}"#;

    const PRICE_KEY: &[u8] = b"price";
    let data = json_val.get_object()[0].1.get_object();

    let (_, v) = data.iter()
      .filter(|(k, _)| PRICE_KEY.to_vec() == Self::vecchars_to_vecbytes(k))
      .nth(0)
      .ok_or("fetch_price_from_gemini: JSON does not conform to expectation")?;

    // `val` contains the price, such as "222.333" in bytes form
    let val_u8: Vec<u8> = v.get_bytes();

    // Convert to number
    let val_f64: f64 = core::str::from_utf8(&val_u8)
      .map_err(|_| "fetch_price_from_gemini: val_f64 convert to string error")?
      .parse::<f64>()
      .map_err(|_| "fetch_price_from_gemini: val_u8 parsing to f64 error")?;
    Ok((val_f64 * 10000.).round() as u64)
  }

  fn aggregate_pp<'a>(block: T::BlockNumber, sym: &'a [u8])
    -> Result<()> {
    let ts_pp_vec = <TokenSrcPPMap<T>>::get(sym);
    let price_sum: u64 = ts_pp_vec.iter().fold(0, |mem, pp| mem + pp.1);

    // Avoiding floating-point arithmetic & do integer division
    let price_avg: u64 = price_sum / (ts_pp_vec.len() as u64);

    // submit onchain call for aggregating the price
    let call = Call::record_agg_pp(block, sym.to_vec(), price_avg);

    // Unsigned tx
    SubmitTransaction::submit_unsigned_transaction(call.into())
      .map_err(|_| "aggregate_pp: submit_unsigned(call) error")

    // Signed tx
    // T::SubmitSignedTransaction::submit_signed(call);
    // Ok(())
  }
}

#[allow(deprecated)]
impl<T: Trait> support::unsigned::ValidateUnsigned for Module<T> {
  type Call = Call<T>;

  #[allow(deprecated)]
  fn validate_unsigned(call: &Self::Call) -> TransactionValidity {

    match call {
      Call::record_price(block, (sym, remote_src, ..), price) => Ok(ValidTransaction {
        priority: 0,
        requires: vec![],
        provides: vec![(block, sym, remote_src, price).encode()],
        longevity: TransactionLongevity::max_value(),
        propagate: true,
      }),
      Call::record_agg_pp(block, sym, price) => Ok(ValidTransaction {
        priority: 0,
        requires: vec![],
        provides: vec![(block, sym, price).encode()],
        longevity: TransactionLongevity::max_value(),
        propagate: true,
      }),
      _ => InvalidTransaction::Call.into()
    }
  }
}
