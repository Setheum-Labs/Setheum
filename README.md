# Setheum Node

Setheum's Blockchain Network node Implementation in Rust, Substrate FRAME and Setheum SERML, ready for hacking :rocket:

<div align="center">
	
[![Setheum version](https://img.shields.io/badge/Setheum-0.8.0-brightgreen?logo=Parity%20Substrate)](https://setheum.xyz/)
[![Substrate version](https://img.shields.io/badge/Substrate-3.0.0-brightgreen?logo=Parity%20Substrate)](https://substrate.dev/)
[![License](https://img.shields.io/github/license/Setheum-Labs/Setheum?color=green)](https://github.com/Setheum-Labs/Setheum/blob/master/LICENSE)
 <br />
[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2FSetheum)](https://twitter.com/Setheum)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/SetheumNetwork)
[![Medium](https://img.shields.io/badge/Medium-gray?logo=medium)](https://medium.com/setheum-labs)
	
</div>

# Introduction
Setheum is founded and initiated and fascilitated by Muhammad-Jibril B.A. who founded Setheum Labs, Setheum Foundation to steward and support the development and advancement of the Network, its ecosystem and its community to foster the development and adoption of decentralised finance by building and supporting cross-chain open finance infrastructure such as the SERP (Setheum Elastic Reserve Protocol) stablecoin system, the SettMint stablecoin minting system and Setheum's built-in payment system SettPay that lets traders and transactors claim cashback (what we call CashDrop) on their transactions to speak of a few. 
Setheum also deploys Advanced Incentivization mechanisms and economic models modeled under the Jurisdiction of Islamic Finance.
Setheum deploys a Multi-Cameral DECENTRALISED GOVERNANCE SYSTEM.

### Founding Members:
* [Setheum Labs](https://github.com/Setheum-Labs)
* [Setheum Foundation](https://github.com/Setheum-Foundation)]
* [Slixon Technologies](https://github.com/Slixon-Technologies)

## Core Products

1. The Setter - The Setter is a basket currency pegged to the Top 10 Strongest and most valuable currencies. It serves as the medium of Exchange and the Defacto stablecoin of the Setheum Ecosystem. All other Setheum system stablecoins orbit around the Setter (SETT) and the SettMint for minting Setheum Currencies (system stablecoins) accepts only the Setter as the Minting Reserve Asset. Only with the Setter (SETT) can a user participate in the DNAR Auctions to stabilize the price of the Setter, while the Setter is Auctioned to stabilize the price of all the other SettCurrencies (system stablecoins). It's the star that brightens many planets - 38 to be exact

2. [The SERP](./lib-serp) - The SERP (Setheum Elastic Reserve Protocol) is algorithmically responsible for stabilizing the prices of the Setheum Stable Currencies. No human interferrance is needed for this, it's all algorithmically handled by the SERP. The SERP is the backbone of Setheum, it is based on my TES (Token Elasticity of Supply) algorithm based on PES (Price Elasticity of Supply) such that the demand curve or price of a currency determines the supply serping point, meaning the supply curve of a SettCurrency will be adjusted according to the demand curve of that specific SettCurrency. The result will be burning or minting an amount equivalent to the serping point produced by the SERP-TES, the burning amount will be bought back by the SERP automatically through the SerpAuction and the bought amount will be burnt to meet the satisfaction of the demand curve to prop the price back up to its peg, the opposite is done to lower the price of an under-supplied currency that is on demand and above its peg on the demand curve, for this the mint amount is divided into receipients including the SettPayTreasury where CashDrops are deposited for users to claim, the System Treasury under Governance, the Charity Fund stewarded by the Setheum Foundation, and the WelfareTreasury, more on the Welfare Treasury below.

In The SERP and Setheum lingua, I coined these terms:
* serp: to increase or decrease the supply of a Setheum stable Currency at its serping point in the curve, on either the x or y axis, the negative or the positive.
* serpup: to increase the supply of a Setheum stablecurrency at its serping point.
* serpdown: to decrease the supply of a Setheum stablecurrency at its serping point.

3. [The SerpAuction](./lib-serml/serp/serp-auction) - The SerpAuction is the Auction House of the SERP, it is the only endpoint of the SERP that interacts with the user. This lets the SERP auction off freshly minted Setheum Currency for Another to burn the latter and set the price stable to its peg. And when a currency is above its peg, as I mentioned abovein the SERP - the SERP System mints that demand-supply gap at the serping point and mints it to the recipients of the SerpUps.

The SerpAuctions are inspired by the Maker Protocol (MakerDAO) Auctions.
There are 2 Types of SerpAuctions:
* Diamond Auctions: A reverse auction to buy back a fixed amount of the Setter (SETT) with a decreasing amount of DNAR.
* Setter Auctions: A reverse auction to buy back a fixed amount of a SettCurrency (except for the Setter ofcourse) with a decreasing amount of Setter (SETT).

4. The WelfareTreasury - The WelfareTreasury is the Treasury that receives some amount of each SerpUp, this amount is then used by the Welfare Council (this council governs the WelfareTreasury) to buy back the DNAR by swapping on the DEX, the Council only has to call and approve the Buy Back, they don't have so much control over the swap. They just can call it and approve it. Then all the DNAR bought back is automatically burnt by the SERP.

5. [The SettMint](./lib-serml/settmint) - The Settmint is partly inspired by the Maker Protocol (MakerDAO), except that SettMint is on a very different principle of Setheum that ought not to be violated.
SettMint is not a CDP but quite similar, as users can hold, authorize & transfer positions, users can reserve the Setter (SETT) to mint any SettCurrency of their choice without the need for over-collateralization, debt, interest rates, liquidation, or even stability fees. The stability of the Currencies is handles by the SERP, and the the Setter used as the reserve currency is also a SettCurrency (Setheum System Stablecoin) therefore eliminating position volatility and the risk of liquidation as all risk parameters have been eliminated with the Setter and Setheum's strong principle on the matters of the SettMint and Setheum's Monetary Policy.
This is one of the reasons I see Setheum as one of the most Sophisticated Advanced Economic Systems yet so simple, easy to use and understand, and even easier to get started.

6. The SettPay - The SettPay is responsible for the CashDrops that are dispatched by the SERP to the claimants (transactors that claim cashdrops). It is under the governance of the "Welfare Council". They decide how much percent claimants get based on how much they spent, these params are custom and governable for every Setheum System currency including the DNAR. For example, DNAR spenders get 2.58% cashdrop per claimed transaction if their spent amount is >= 10000 dollars, else if their spent amount is < 10000 dollars && >= 100 then they get 4% cashdrop, else if their spent amount is < 100 then they get 5% cashdrop. The Welfare Council can update these parameters by governance proposals and voting without the need for forking or even a runtime upgrade.

3. [The SEVM](./lib-serml/evm) - The Setheum EVM is an Ethereum Virtual Machine (EVM) compatibility layer that implements the EVM on Setheum and bridges to Ethereum that opens the ground for interoperability between Ethereum and Setheum.
The SEVM lets developers onboard, deploy or migrate their Ethereum Solidity Smart Contracts on Setheum seamlessly with little to no change in their code.
The SetheumEVM has a beautiful library of developer tools that let developers deploy, manageand interact with their smart contracts and upgradable smart contracts on the S-EVM with popular and well documented tools like Truffle, MetaMask, et al.
The Setters.JS is the Web3 Ethers.JS compatibility library for the Setheum EVM, to let users access the Setheumand the EVM both with a single wallet without having to use two separate wallets for compatibility.

For all the SERML (Setheum Runtime Module Library) modules like the;
[bridges](./lib-serml/bridges), 
[dex](./lib-serml/dex), 
[prices](./lib-serml/serp/serp-prices), 
[support](./lib-serml/support), 
[tokens](./lib-serml/tokens), 
[NFTs](./lib-serml/nft), 
[transaction-payment](./lib-serml/transaction-payment) 
et al.
Check the [lib-serml](./lib-serml)

# Getting Started 

This project contains some configuration files to help get started :hammer_and_wrench:

### Rust Setup

Follow the [Rust setup instructions](./doc/rust-setup.md) before using the included Makefile to
build the Setheum node.

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Make sure you have `submodule.recurse` set to true to configure submodule.

```bash
git config --global submodule.recurse true
```

Install required tools and install git hooks:

```bash
make init
```

Build all native code:

```bash
make build
```

## Run

You can start a development chain with:

```bash
make run
```

## Development

To type check:

```bash
make check-all
```

To purge old chain data:

```bash
make purge
```

To purge old chain data and run

```bash
make restart
```

Update ORML

```bash
make update
```

__Note:__ All build command from Makefile are designed for local development purposes and hence have `SKIP_WASM_BUILD` enabled to speed up build time and use `--execution native` to only run use native execution mode.

## 6. Bench Bot
Bench bot can take care of syncing branch with `master` and generating WeightInfos for module or runtime.

### Generate module weights

Comment on a PR `/bench runtime module <setheum_name>` i.e.: `serp_prices`

Bench bot will do the benchmarking, generate `weights.rs` file push changes into your branch.

### Generate runtime weights

Comment on a PR `/bench runtime <runtime> <setheum_name>` i.e.: `/bench runtime newrome setheum_currencies`.

To generate weights for all modules just pass `*` as `setheum_name` i.e: `/bench runtime newrome *`

Bench bot will do the benchmarking, generate weights file push changes into your branch.
