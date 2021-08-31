# Setheum Node

Setheum's Blockchain Network node Implementation in Rust, Substrate FRAME and Setheum SERML, ready for hacking :rocket:

<div align="center">
	
[![Setheum version](https://img.shields.io/badge/Setheum-0.9.0-brightgreen?logo=Parity%20Substrate)](https://setheum.xyz/)
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

### The Tokens/Currencies

#### The Main Coins - Tokens
```
DNAR("Setheum Dinar", 12) = 0, // Staking and Utility Token - NativeCurrency & Reserve Asset - System GoldenCurrency
DRAM("Setheum Dirham", 12) = 1, // Staking Reward and Governance Token - System SilverCurrency
```

#### The Setter - SERP Basket Stablecoin
```
		SETR("Setter", 12) = 2,
```

#### The SettCurrencies - SERP Stablecoins
```
		SETUSD("SetDollar", 12) = 3,
		SETEUR("SetEuro", 12) = 4,
		SETGBP("SetPound", 12) = 5,
		SETCHF("SetFranc", 12) = 6,
 		SETSAR("SetRiyal", 12) = 7,
```

1. The Setter - The Setter is a basket currency pegged to the Top 10 Strongest and most valuable currencies. It serves as the medium of Exchange and the Defacto stablecoin of the Setheum Ecosystem. All other Setheum system stablecoins orbit around the Setter (SETR) and the SettMint for minting Setheum Currencies (system stablecoins) accepts only the Setter as the Minting Reserve Asset. Only with the Setter (SETR) can a user participate in the DNAR Auctions to stabilize the price of the Setter, while the Setter is Auctioned to stabilize the price of all the other SettCurrencies (system stablecoins). It's the star that brightens many planets - 10 to be exact

2. [The SERP](./lib-serp) - The SERP (Setheum Elastic Reserve Protocol) is algorithmically responsible for stabilizing the prices of the Setheum Stable Currencies. No human interferrance is needed for this, it's all algorithmically handled by the SERP. The SERP is the backbone of Setheum, it is based on my TES (Token Elasticity of Supply) algorithm based on PES (Price Elasticity of Supply) such that the demand curve or price of a currency determines the supply serping point, meaning the supply curve of a SetCurrency will be adjusted according to the demand curve of that specific SetCurrency. The result will be burning or minting an amount equivalent to the serping point produced by the SERP-TES, the burning amount will be bought back by the SERP automatically through the built-in-DEX and the bought amount will be burnt to meet the satisfaction of the demand curve to prop the price back up to its peg, the opposite is done to lower the price of an under-supplied currency that is on demand and above its peg on the demand curve, for this the mint amount is divided into receipients including the SettPayTreasury where CashDrops are deposited for users to claim, the System Treasury under Governance, the Charity Fund stewarded by the Setheum Foundation, and the WelfareTreasury, more on the Welfare Treasury below.

In The SERP and Setheum lingua, I coined these terms:
* serp: to increase or decrease the supply of a Setheum stable Currency at its serping point in the curve, on either the x or y axis, the negative or the positive.
* serpup: to increase the supply of a Setheum stablecurrency at its serping point.
* serpdown: to decrease the supply of a Setheum stablecurrency at its serping point.

3. [The SettMint](./lib-serml/settmint) - The Settmint is partly inspired by the Maker Protocol (MakerDAO), except that SettMint is on a very different principle of Setheum that ought not to be violated.
SettMint is not a CDP but quite similar, as users can hold, authorize & transfer positions, users can reserve the Setter (SETR) to mint any SetCurrency of their choice without the need for over-collateralization, debt, interest rates, liquidation, or even stability fees. The stability of the Currencies is handles by the SERP, and the the Setter used as the reserve currency is also a SetCurrency (Setheum System Stablecoin) therefore eliminating position volatility and the risk of liquidation as all risk parameters have been eliminated with the Setter and Setheum's strong principle on the matters of the SettMint and Setheum's Monetary Policy.
This is one of the reasons I see Setheum as one of the most Sophisticated Advanced Economic Systems yet so simple, easy to use and understand, and even easier to get started.

4. The SettPay - The SettPay is responsible for the CashDrops that are dispatched by the SERP to the claimants (transactors that claim cashdrops). It is under the governance of the "Financial Council". They decide how much percent claimants get based on how much they spent, these params are custom and governable for every Setheum System currency including the DNAR. For example, DNAR spenders get 2.58% cashdrop per claimed transaction if their spent amount is >= 10000 dollars, else if their spent amount is < 10000 dollars && >= 100 then they get 4% cashdrop, else if their spent amount is < 100 then they get 5% cashdrop. The Welfare Council can update these parameters by governance proposals and voting without the need for forking or even a runtime upgrade.

5. [The SEVM](./lib-serml/evm) - The Setheum EVM is an Ethereum Virtual Machine (EVM) compatibility layer that implements the EVM on Setheum and bridges to Ethereum that opens the ground for interoperability between Ethereum and Setheum.
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

## Initialisation

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
./scripts/init.sh submodule build-full

```

## Build
Build all native code:

### Build NewRome (Testnet) - `SKIP_WASM_BUILD`
```bash
SKIP_WASM_BUILD= cargo build --features with-newrome-runtime
```

### Build Full NewRome (Testnet)
```bash
cargo build --features with-newrome-runtime
```

### Build All Runtimes (Testnet + Mainnet)
```bash
cargo build --locked --features with-all-runtime
```

## Check
Check all native code:

### Check NewRome (Testnet)
```bash
SKIP_WASM_BUILD= cargo check --features with-newrome-runtime
```

### Check Setheum (Mainnet)
```bash
SKIP_WASM_BUILD= cargo check --features with-setheum-runtime
```

### Check All Tests (Testnet + Mainnet)
```bash
SKIP_WASM_BUILD= cargo check --features with-all-runtime --tests --all
```

### Check All Runtimes (Testnet + Mainnet)
```bash
SKIP_WASM_BUILD= cargo check --features with-all-runtime --tests --all
```

### Check All Benchmarks (Testnet + Mainnet)
```bash
SKIP_WASM_BUILD= cargo check --features runtime-benchmarks --no-default-features --target=wasm32-unknown-unknown -p newrome-runtime
SKIP_WASM_BUILD= cargo check --features runtime-benchmarks --no-default-features --target=wasm32-unknown-unknown -p setheum-runtime
```

### Check All Runtimes & Benchmarks (Testnet + Mainnet)
```bash
SKIP_WASM_BUILD= cargo check --features with-all-runtime --tests --all
SKIP_WASM_BUILD= cargo check --features runtime-benchmarks --no-default-features --target=wasm32-unknown-unknown -p newrome-runtime
SKIP_WASM_BUILD= cargo check --features runtime-benchmarks --no-default-features --target=wasm32-unknown-unknown -p setheum-runtime
```

### Check Debug (Testnet)
```bash
RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-newrome-runtime
```

### Check `try-runtime` (Testnet + Mainnet)
```bash
SKIP_WASM_BUILD= cargo check --features try-runtime --features with-all-runtime
```

## Test
Test all native code:

### Test (Testnet)
```bash
SKIP_WASM_BUILD= cargo test --features with-newrome-runtime --all
```

### Test setheum-evm (SEVM Testnet)
```bash
SKIP_WASM_BUILD= cargo test --all --features with-ethereum-compatibility test_setheum_evm
SKIP_WASM_BUILD= cargo test --all --features with-ethereum-compatibility should_not_kill_contract_on_transfer_all
SKIP_WASM_BUILD= cargo test --all --features with-ethereum-compatibility schedule_call_precompile_should_work
SKIP_WASM_BUILD= cargo test --all --features with-ethereum-compatibility schedule_call_precompile_should_handle_invalid_input
```

### Test All Runtimes (Testnet + Mainnet)
```bash
SKIP_WASM_BUILD= cargo test --all --features with-all-runtime
```

### Test All Benchmarking (Testnet + Mainnet)
```bash
cargo test --features runtime-benchmarks --features with-all-runtime --features --all benchmarking
```

### Test All - Runtimes, SEVM, Benchmarking (Testnet + Mainnet)
```bash
SKIP_WASM_BUILD= cargo test --all --features with-all-runtime
SKIP_WASM_BUILD= cargo test --all --features with-ethereum-compatibility test_setheum_evm
SKIP_WASM_BUILD= cargo test --all --features with-ethereum-compatibility should_not_kill_contract_on_transfer_all
SKIP_WASM_BUILD= cargo test --all --features with-ethereum-compatibility schedule_call_precompile_should_work
SKIP_WASM_BUILD= cargo test --all --features with-ethereum-compatibility schedule_call_precompile_should_handle_invalid_input
cargo test --features runtime-benchmarks --features with-all-runtime --features --all benchmarking
```

## Update

### Update Cargo
```bash
cargo update
```

### Update ORML
```bash
cd lib-openrml && git checkout master && git pull
git add lib-openrml
cargo-update check-all
```

## Development (NewRome Dev)

### Run - NewRome (Dev Chain)
You can start a development chain with:

```bash
cargo run --features with-newrome-runtime -- --dev -lruntime=debug --instant-sealing
```

### Run - SEVM Development (Dev Chain - NewRome)
You can start a development chain with:

```bash
cargo run --features with-newrome-runtime --features with-ethereum-compatibility -- --dev -lruntime=debug -levm=debug --instant-sealing
```

### Purge - Development (Dev Chain)
```bash
target/debug/setheum purge-chain --dev -y
```

### Restart - Development (Dev Chain - NewRome)
```bash
target/debug/setheum purge-chain --dev -y
cargo run --features with-newrome-runtime -- --dev -lruntime=debug --instant-sealing
```

### Restart - Development (Dev Chain - NewRome)
You can start a development chain with:

```bash
cargo run --features with-newrome-runtime -- --dev -lruntime=debug --instant-sealing
```

## TestNet (NewRome)

### Build NewRome WASM

```bash
./scripts/build-only-wasm.sh -p newrome-runtime --features=with-ethereum-compatibility
```

## Mainnet (Setheum - `Not Production Ready!`)

### Build Setheum WASM

```bash
./scripts/build-only-wasm.sh -p setheum-runtime --features=on-chain-release-build
```

### Build Setheum WASM (srtool)

```bash
PACKAGE=setheum-runtime BUILD_OPTS="--features on-chain-release-build" ./scripts/srtool-build.sh
```

### Generate Tokens & Predeploy Contracts - SetheumEVM (SEVM)

```bash
./scripts/generate-tokens-and-predeploy-contracts.sh
```

__Note:__ All build commands with `SKIP_WASM_BUILD` are designed for local development purposes and hence have the `SKIP_WASM_BUILD` enabled to speed up build time and use `--execution native` to only run use native execution mode.

## 6. Bench Bot
Bench bot can take care of syncing branch with `master` and generating WeightInfos for module or runtime.

### Generate module weights

Comment on a PR `/bench runtime module <setheum_name>` i.e.: `serp_prices`

Bench bot will do the benchmarking, generate `weights.rs` file push changes into your branch.

### Generate runtime weights

Comment on a PR `/bench runtime <runtime> <setheum_name>` i.e.: `/bench runtime newrome setheum_currencies`.

To generate weights for all modules just pass `*` as `setheum_name` i.e: `/bench runtime newrome *`

Bench bot will do the benchmarking, generate weights file push changes into your branch.
