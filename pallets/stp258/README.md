# Currencies Module

## Overview

The STP258 Currencies Pallet provides a mixed stablecoin currencies system, by configuring a native currency which implements `BasicCurrencyExtended`, and a multi-currency for stable-currencies which implements `SettCurrency`.

It also provides a system to receive newly minted stable-currency automatically, that implements the `shares` and the `shareholders` who hold these shares to receive the the sett-currencies.
It also provides an adapter, to adapt `frame_support::traits::Currency` implementations into `BasicCurrencyExtended`.

The STP258 Currencies Pallet provides functionality of both `SettCurrencyExtended` and `BasicCurrencyExtended`, via unified interfaces, and all calls would be delegated to the underlying multi-currency and base currency system. A native currency ID could be set by `Config::GetNativeCurrencyId`, to identify the native currency.
