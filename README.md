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

Setheum is founded and initiated and fascilitated by Muhammad-Jibril B.A. of Setheum Labs, Setheum Foundation and Slixon Technologies to steward and support the development and advancement of the Network, its ecosystem and its community to foster the development and adoption of decentralised finance by building and supporting cross-chain open finance infrastructure.
Setheum also deploys Advanced Incentivization mechanisms and economic models modeled under the Jurisdiction of Islamic Finance.

### Founder/Developer:

* [Muhammad-Jibril B.A.](https://github.com/JBA-Khalifa)

## Core Products - Mostly Unique to Setheum

### The Tokens


```bash
    SETM("Setheum", 12) = 0,
    SLIX("Slixon", 12) = 1,
    USDI("InterUSD", 12) = 2,
    USDW("WestUSD", 12) = 3,
```

1. [The SEVM](./lib-serml/sevm) - The Setheum EVM is an Ethereum Virtual Machine (EVM) compatibility layer that implements the EVM on Setheum and bridges to Ethereum that opens the ground for interoperability between Ethereum and Setheum.
The SEVM lets developers onboard, deploy or migrate their Ethereum Solidity Smart Contracts on Setheum seamlessly with little to no change in their code.
The SetheumEVM has a beautiful library of developer tools that let developers deploy, manageand interact with their smart contracts and upgradable smart contracts on the S-EVM with popular and well documented tools like Truffle, MetaMask, et al.
The Setters.JS is the Web3 Ethers.JS compatibility library for the Setheum EVM, to let users access the Setheum and the EVM both with a single wallet without having to use two separate wallets for compatibility.

2. [The SetMint](./lib-serml/defi/setmint) - Inspired by MakerDAO Protocol, the CDP (Collateralized Debt Position) protocol on Ethereum. The Setheum CDP has zero interest rates, zero stability fees, and is fully halal and collateralized. This differentiates SetMint from any other CDP Protocol, making it by far the only halal loan protocol in the entire industry. And it is Multi-Collateral.
Just reserve some collateral to mint some USDI/USDW, when returning the loan just return exactly what was loaned and unreserve the collateral with no fees and no interest.
This lets the muslim world also participate in the industry and take part in trading and yield making strategies that are within their dome of principles, for me this is a gamechanger that I wished was there for me, therefore I am building it for people like me who need it but haven’t been given the chance to be pleased by it, and also non-muslims that want to break-free from the interest-based alternatives to a more efficient system based on truth, fairness and equality.

3. [The LaunchPad](./lib-serml/defi/launchpad) - The Setheum Help LaunchPad is a crowdfunding protocol for teams & projects to raise soney and launch their tokens to the public and add their tokens to the Setheum DEX (SetSwap). It provides halal incentives to LPs to provide liquidity for token launches to bootstrap on the SetSwap and lets the teams/projects raise funds while getting help listing bootstrap pool on DEX. Governed by the` LaunchPadCouncil`

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

### Fork Setheum

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
