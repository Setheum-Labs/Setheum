# Currencies Module

## Overview

This is the `SerpMarket` Pallet that trades with the SERP system 
to make bids for Nativecurrency in this case called Dinar, and Sett-Currencies(Multiple stablecoins).
Dutch Auction for Bids on stability of the Stablecoins.

 Then to determine whether the SERP should expand or contract supply, the TES provides
 a `serp_elast` to tell the TES when to expand and when to contract supply depending on 
 the outcome of the price of the stablecoin.

 It also constructs a transient storage adapter for the bids priority queue and stores the Bids in a Bonded Priority Queue.

 The `SerpMarket` module makes trade/auction of DNAR and the stable settcurrencies minted and 
 contracted by the `SerpTes` module.
 
 The `SerpMarket` module depends on the `FetchPrice` module to feed the prices of the 
 currencies in to adjust the stablecoin supply.

## Acknowledgement

This Pallet is built with the [Stablecoin](https://github.com/apopiak/stablecoin) Pallet originally developed by [Alexander Popiak](https://github.com/apopiak), for reference check [The Apopiak/Stablecoin Repo](https://github.com/apopiak/stablecoin).