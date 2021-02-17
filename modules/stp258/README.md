# stp258-serp-module
STP258 Multi-Currency Stablecoin Pallet -- Setheum Tokenization Protocol 258

## Overview

The STP258 Currencies Pallet provides a mixed stablecoin currencies system, by configuring a native currency which implements `ExtendedBasicCurrency`, and a multi-currency for stable-currencies which implements `SettCurrency`.

It also provides a system to receive newly minted stable-currency automatically, that implements the `shares` and the `shareholders` who hold these shares to receive the the sett-currencies.
It also provides an adapter, to adapt `frame_support::traits::Currency` implementations into `ExtendedBasicCurrency`.

The STP258 Currencies Pallet provides functionality of both `ExtendedSettCurrency` and `ExtendedBasicCurrencyExtended`, via unified interfaces, and all calls would be delegated to the underlying multi-currency and base currency system. A native currency ID could be set by `Config::GetNativeCurrencyId`, to identify the native currency.

## Acknowledgement

This Pallet is built with the [ORML Currencies](https://github.com/open-web3-stack/open-runtime-module-library/blob/master/currencies) Pallet originally developed by [Open Web3 Stack](https://github.com/open-web3-stack/), for reference check [The ORML Repo](https://github.com/open-web3-stack/open-runtime-module-library).

This Pallet is built with the [Stablecoin](https://github.com/apopiak/stablecoin) Pallet originally developed by [Alexander Popiak](https://github.com/apopiak), for reference check [The Apopiak/Stablecoin Repo](https://github.com/apopiak/stablecoin).