بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

# Setheum Node

Setheum's Blockchain Network node Implementation in Rust, Substrate FRAME and Setheum SERML, ready for hacking :rocket:

<div align="center">
	
[![Setheum version](https://img.shields.io/badge/Setheum-1.0.0-brightgreen?logo=Parity%20Substrate)](https://setheum.xyz/)
[![Substrate version](https://img.shields.io/badge/Substrate-4.0.0-brightgreen?logo=Parity%20Substrate)](https://substrate.dev/)
[![License](https://img.shields.io/github/license/Setheum-Labs/Setheum?color=green)](https://github.com/Setheum-Labs/Setheum/blob/master/LICENSE)
 <br />

[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2FSetheum)](https://twitter.com/Setheum)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/SetheumNetwork)
[![Medium](https://img.shields.io/badge/Medium-gray?logo=medium)](https://medium.com/setheum-labs)
	
</div>

# Introduction

Setheum is founded and initiated and fascilitated by Muhammad-Jibril B.A. of Setheum Labs, Setheum Foundation and Slixon Technologies to steward and support the development and advancement of the Network, its ecosystem and its community to foster the development and adoption of decentralised finance by building and supporting cross-chain open finance infrastructure such as the SERP (Setheum Elastic Reserve Protocol) stablecoin system, the SetMint stablecoin minting system and Setheum's built-in payment system SetPay(CashDrops) that lets traders and transactors claim cashback (what we call CashDrop) on their transactions to speak of a few. 
Setheum also deploys Advanced Incentivization mechanisms and economic models modeled under the Jurisdiction of Islamic Finance.

### Founder/Developer:

* [Muhammad-Jibril B.A.](https://github.com/JBA-Khalifa)

## Core Products - Mostly Unique to Setheum

### The Tokens - [Currencies](./primitives/src/currency.rs#:~:text=%7D-,create_currency_id!%20%7B,%7D,-pub%20trait%20TokenInfo)


```bash
    // Tier-1 Tokens
    SETM("Setheum", 12) = 0,
    SERP("Serp", 12) = 1,
    DNAR("The Dinar", 12) = 2,
    HELP("HighEnd LaunchPad", 12) = 3,
    // Tier-2 Tokens (StableCurrencies)
    SETR("Setter", 12) = 4,
    SETUSD("SetDollar", 12) = 5,
```

1. [The Setter](./primitives/src/currency.rs#:~:text=SETR(%22Setter%22%2C%2012)%20%3D%203%2C) - The Setter is a stable currency pegged to the US dollar at a ratio of 1:10, where 1 SETR = $0.25 USD (25 cents or $1 USD = 4 SETRs).

2. [The SERP](./lib-serml/serp-treasury) - The SERP (Setheum Elastic Reserve Protocol) is algorithmically responsible for stabilizing the prices of the Setheum Stable Currencies. No human interferrance is needed for this, it's all algorithmically handled by the SERP. The SERP is the backbone of Setheum, it is based on my TES (Token Elasticity of Supply) algorithm based on PES (Price Elasticity of Supply) such that the demand curve or price of a currency determines the supply serping point, meaning the supply curve of a SetCurrency will be adjusted according to the demand curve of that specific SetCurrency. The result will be burning or minting an amount equivalent to the serping point produced by the SERP-TES, the burning amount will be bought back by the SERP automatically through the built-in-DEX and the bought amount will be burnt to meet the satisfaction of the demand curve to prop the price back up to its peg, the opposite is done to lower the price of an under-supplied currency that is on demand and above its peg on the demand curve, for this the mint amount is divided into receipients including the SetPayTreasury where CashDrops are deposited for users to claim, the System Treasury under Governance, the Charity Fund stewarded by the Setheum Foundation, and the WelfareTreasury, more on the Welfare Treasury below.

	In The SERP and Setheum lingua, I coined these terms:
	* serp: to increase or decrease the supply of a Setheum stable Currency at its serping point in the curve, on either the x or y axis, the negative or the positive.
	* serpup: to increase the supply of a Setheum stablecurrency at its serping point.
	* serpdown: to decrease the supply of a Setheum stablecurrency at its serping point.

3. [The CashDrops](./lib-serml/currencies/src/lib.rs#:~:text=T%3A%3ASerpTreasury%3A%3Aclaim_cashdrop(currency_id%2C%20%26from%2C%20amount)%3F) - The CashDrops that are dispatched by the SERP to the claimants (transactors/transactions that claim cashdrops). Whoever transacts witha SetCurrency (Setheum Stable Currency), they can toggle on claim cashdrop to get a cashback of (2% for SETUSD and/or 4% for SETR) of the amount they transferred/sent, this amount is only cashdropped if the CashDropPool has enough funds to cover that amount. The funds in the CashDropPool are provided by the SERP through SerpUps and through SerpTransactionFees. SerpTransactionFees are 0.2% transaction fees that are paid on SetCurrency transactions, these funds are then transferred by the system to the SERP'S CashDropPool for CashDrops ready to be claimed.

4. [The EFE and EFFECTs](./lib-serml/dex/src/lib.rs#:~:text=type%20GetStableCurrencyExchangeFee%3A%20Get%3C(u32%2C%20u32)%3E%3B) - The EFE is the ExchangeFeeEvaluator, it basically takes lower exchange fees than the normal rate on DEX for stablecurrency pools, and the difference between those two rates is settled by the EFE to the LPs in stablecurrencies, to be eligible for the EFE one of the two currencies in the LP pair must be a Setheum stablecurrency. Therefore, the traders of that pool pay less fees and the LPs get more income because these pairs will attract more traders and that will in turn attract more liquidity by the LPs.

5. [The SEVM](./lib-serml/evm) - The Setheum EVM is an Ethereum Virtual Machine (EVM) compatibility layer that implements the EVM on Setheum and bridges to Ethereum that opens the ground for interoperability between Ethereum and Setheum.
The SEVM lets developers onboard, deploy or migrate their Ethereum Solidity Smart Contracts on Setheum seamlessly with little to no change in their code.
The SetheumEVM has a beautiful library of developer tools that let developers deploy, manageand interact with their smart contracts and upgradable smart contracts on the S-EVM with popular and well documented tools like Truffle, MetaMask, et al.
The Setters.JS is the Web3 Ethers.JS compatibility library for the Setheum EVM, to let users access the Setheumand the EVM both with a single wallet without having to use two separate wallets for compatibility.

6. [The SetMint](./lib-serml/serp-setmint) - Inspired by MakerDAO Protocol, the CDP (Collateralized Debt Position) protocol on Ethereum. The Setheum CDP has zero interest rates, zero stability fees, and is fully halal and collateralized. This differentiates SetMint from any other CDP Protocol, making it by far the only halal loan protocol in the entire industry. And it is Multi-Collateral.
Just reserve some collateral to mint some SETUSD, when returning the loan just return exactly what was loaned and unreserve the collateral with no fees and no interest.
This lets the muslim world also participate in the industry and take part in trading and yield making strategies that are within their dome of principles, for me this is a gamechanger that I wished was there for me, therefore I am building it for people like me who need it but haven’t been given the chance to be pleased by it, and also non-muslims that want to break-free from the interest-based alternatives to a more efficient system based on truth, fairness and equality.

6. [The HELP (High Engagement LaunchPad)](./lib-serml/help-launchpad) - The Setheum Help LaunchPad is a crowdfunding protocol for teams & projects to raise soney and launch their tokens to the public and add their tokens to the Setheum DEX (SetSwap). It provides halal incentives to LPs to provide liquidity for token launches to bootstrap on the SetSwap and lets the teams/projects raise funds while getting help listing bootstrap pool on DEX. Governed by the` LaunchPadCouncil`

For all the SERML (Setheum Runtime Module Library) modules Check the [lib-serml](./lib-serml)

# Getting Started 

This project contains some configuration files to help get started :hammer_and_wrench:

## Initialisation

Clone this repository:

```bash
git clone --recursive https://github.com/Setheum-Labs/Setheum
```

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

> If, after installation, running `rustc --version` in the console fails, refer to [it](https://www.rust-lang.org/tools/install) to repair.

You can install developer tools on Ubuntu 20.04 with:

```bash
sudo apt install make clang pkg-config libssl-dev build-essential
```

You can install the latest Rust toolchain with:

```bash
make init
```

Make sure you have `submodule.recurse` set to true to configure submodule.

```bash
git config --global submodule.recurse true
```

Install required tools and install git hooks:

```bash
git submodule update --init --recursive
```

### Start a development node

The `make run` command will launch a temporary node and its state will be discarded after you terminate the process.
```bash
make run
```

### Run a persistent single-node chain

Use the following command to build the node without launching it:

```bash
make build
```

This command will start the single-node development chain with persistent state:

```bash
./target/release/setheum-node --dev
```

Purge the development chain's state:

```bash
./target/release/setheum-node purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/setheum-node -lruntime=debug --dev
```

### Run tests

```bash
make test
```

### Run benchmarks

Run runtime benchmark tests:
```bash
make bench
```

Run module benchmark tests:
```bash
cargo test -p module-poc --all-features
```

Run the module benchmarks and generate the weights file:
```
./target/release/setheum-node benchmark \
    --chain=dev \
    --steps=50 \
    --repeat=20 \
    --pallet=module_currencies \
    --extrinsic='*'  \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output=./lib-serml/currencies/src/weights.rs
```

### Run in debugger

```bash
make debug
```

### Nodes

For Docs on running nodes, check [./docs/nodes.md](./docs/nodes.md)

### Embedded docs

Once the project has been built, the following command can be used to explore all parameters and subcommands:

```bash
./target/release/setheum-node -h
```

### Release builds

To list all available release builds run:
```bash
git tag
```

To create a corresponding production build, first checkout the tag:
```bash
git checkout testnet-1
```

Then run this command to install appropriate compiler version and produce a binary.
```bash
make release
```

### On-Chain upgrade builds

Build the wasm runtime with:
```bash
make wasm
```

## Update

### Update Cargo

```bash
make update
```

### Update ORML

```bash
cd lib-orml && git checkout master && git pull
git add lib-orml
cargo update check-all
```

### Update Predeploy-Contracts

```bash
cd lib-serml/sevm/predeploy-contracts && git checkout master && git pull
git add predeploy-contracts
cargo update check-all
```

### Generate Tokens & Predeploy Contracts - SetheumEVM (SEVM)

```bash
make generate-tokens
```

__Note:__ All build commands with `SKIP_WASM_BUILD` are designed for local development purposes and hence have the `SKIP_WASM_BUILD` enabled to speed up build time and use `--execution native` to only run use native execution mode.

## Bench Bot

Bench bot can take care of syncing branch with `master` and generating WeightInfos for module or runtime.

### Generate Module weights

#### Generate Weights on Git with PR

Comment on a PR `/bench runtime module <setheum_module_name>` i.e.: `serp_prices`

Bench bot will do the benchmarking, generate `weights.rs` file push changes into your branch.

#### Generate Runtime Module Weights Locally

```bash
make benchmark
```

### Fork setheum-chain

You can create a fork of a live chain (testnet / mainnet) for development purposes.

1) Build binary and sync with target chain on localhost defaults. You will need to use unsafe rpc.
2) Execute the `Make` command ensuring to specify chain name (testnet / mainnet).
```bash
make chain=testnet fork
```
3) Now run a forked chain:
```bash
cd fork/data
./binary --chain fork.json --alice
```
