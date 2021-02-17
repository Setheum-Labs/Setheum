# Currencies Module

## Overview

 The SERP-TES (Setheum Elastic Reserve Protocol - Token Elasticity of Supply) 
 module provides a token elasticity system for the SERP-STP258 mixed stablecoin system, 
 by configuring an expansion which implements an `expand_supply` to expand stablecoin supply
 and a `contract_supply` which contracts the stablecoin supply.

 Then to determine whether the SERP should expand or contract supply, the TES provides
 a `serp_elast` to tell the TES when to expand and when to contract supply depending on 
 the outcome of the price of the stablecoin.

 It also provides a `hand_out_settcurrency` that implements an adapter to hand out the 
 newly minted stablecoin to the shareholders of the SERP.
 TODO!: Also handout 25% of each `expand_supply` to the stakers of the network to buy back NativeCurrency through SerpMarket - as per the
 Setheum white paper (Do, When Staking Pallet is Added).

 The serp-tes module provides functionality of both the `Stp258` module that needs 
 to contract and expand the supply of its currencies for its stablecoin system stability 
 and the `SerpMarket` module that needs to trade/auction the currencies minted and 
 contracted by the `SerpTes` module, which it has to do with the `SerpStaking` module to be 
 built in the next Milestone of the Serp Modules.
 
 The `SerpTes` module depends on the `FetchPrice` module to feed the prices of the 
 currencies in to adjust the stablecoin supply.

## Acknowledgement

This Pallet is built with the [Stablecoin](https://github.com/apopiak/stablecoin) Pallet originally developed by [Alexander Popiak](https://github.com/apopiak), for reference check [The Apopiak/Stablecoin Repo](https://github.com/apopiak/stablecoin).