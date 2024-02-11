بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

# Setheum - Powering The New Internet

<p align="center">
  <img src="./media/SetheumLabel.jpg" style="width:1300px" />
</p>

* Decentralized
* Exceptional
* Secure
* Interoperable
* Reliable
* Ethical
* Scalable

Setheum's Blockchain Network node Implementation in Rust, ready for hacking :rocket:

<div align="center">

[![Setheum version](https://img.shields.io/badge/Setheum-1.0.0-blue?logo=Parity%20Substrate)](https://setheum.xyz/)
[![Substrate version](https://img.shields.io/badge/Substrate-4.0.0-brightgreen?logo=Parity%20Substrate)](https://substrate.dev/)
[![License](https://img.shields.io/github/license/Setheum-Labs/Setheum?color=blue)](https://github.com/Setheum-Labs/Setheum/blob/master/LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](docs/contributor/CONTRIBUTING.md)

 <br />

[![Website](https://img.shields.io/badge/web-gray?logo=web)](https://setheum.xyz)
[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2FSetheum)](https://twitter.com/Setheum)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/SetheumNetwork)
[![Medium](https://img.shields.io/badge/Medium-gray?logo=medium)](https://medium.com/setheum-labs)

</div>

<!-- TOC -->

- [1. Introduction](#1-introduction)
- [2. Setheum](#2-Setheum)
  - [2.1. aUSD and the Honzon stablecoin protocol](#21-ausd-and-the-honzon-stablecoin-protocol)
  - [2.2. Acala Network Economic Model](#22-acala-network-economic-model)
- [3. Building](#3-building)
- [4. Run](#4-run)
- [5. Development](#5-development)
- [6. Bug Bounty :bug:](#6-bug-bounty-bug)

<!-- /TOC -->

## 1. Introduction

> NOTE: SETHEUM means `Salam Ethereum`, it also means `The house of gifts` from the name `Seth/Sheeth` meaning `gift` in hebrew and the name of the Prophet Sheeth/Seth in Islam,  it also stands for `Secure, Evergreen, Truthful, Heterogeneous, Economically Unbiased Market`.

### 1.1 Setheum

Founded November 2019,Setheum achieves a high level of equilibrium in the trilemma by leveraging a Directed Acyclic Graph(DAG) to build the blockchain consensus
making it a Blockchain via DAG, achieve instant finality, high throughput and very fast blocktime while preserving network security and having a fairly decentralised network,

Setheum is a secure, confidential and interoperable decentralised internet cloud compute and storage blockchain network with EVM and WASM smart contracts,
web3 and web 2 Support. The intent of the Setheum Network is to improve upon Web3 and solve the blockchain trilemma with a mixture of approaches and a recipe
formed from what we have seen and considered to be some of the best solutions in the field, improving on scalability, security, mass adoption, diversity,
and ethics while preserving decentralisation and democratisation.

etheum intends to be the most scalable blockchain network in the world while providing
confidentiality for smart contracts, Cloud Computing and Storage Infrastructure for Web3 based Internet Solutions and Interoperability with both Web2 and
other Web3 Networks. The AlephBFT Consensus Engine powers the Setheum Chain to have near instant finality,
high throughput and high scalability.

Setheum’s consensus system works to achieve high scalability and high security with an ethical and equitably high level of decentralisation.

### 1.2 EthicalDeFi

EthicalDeFi Suite is the DeFi powerhouse of the Setheum Network, providing all kinds of top notch DeFi protocols including a cutting-edge AMM DEX, modules,
Decentralised Liquid Staking for Setheum SE and ethical zero-interest halal stablecoins that gives us the properties of both Fiat and Crypto with SlickUSD (USSD)
and the Setter (SETR) using an Ethical Collateralized Debt Position (ECDP) mechanism that is over-Collateralized and multi-Collateralised and stable
without compromising decentralisation or economic stability, offering zero-interest loans of stable cryptocurrencies that has scalable value and trust,
setheum provides just that, backed by crypto assets with efficient zero-interest loans.

## 2. Getting Started

This project contains some configuration files to help get started :hammer_and_wrench:

### 2.1 Build

Clone this repository:

```bash
git clone --recursive https://github.com/Setheum-Labs/Setheum
```

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

> If, after installation, running `rustc --version` in the console fails, refer to [it](https://www.rust-lang.org/tools/install) to repair.

You can install developer tools on Ubuntu with:

```bash
sudo apt-get install -y git make clang curl pkg-config libssl-dev llvm libudev-dev protobuf-compiler build-essential
```

You may need additional dependencies, checkout [substrate.io](https://docs.substrate.io/v3/getting-started/installation) for more info.

Make sure you have `submodule.recurse` set to true to configure submodules.

```bash
git config --global submodule.recurse true
```

You can install required tools and git hooks:

```bash
make init
```

<!-- 
Build Qingdao Testnet native code:

```bash
make build-full
```
 -->

### 2.2. Run

#### 2.2.1 Start a development node

The `make run` command will launch a temporary node and its state will be discarded after you terminate the process.

```bash
make run
```

#### 2.2.2 Run a persistent single-node chain

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

## 3. Development

Run type check:

```bash
make check-all
```

Run tests:

```bash
make test
```

Run in debugger:

```bash
make debug
```

Purge old chain data:

```bash
make purge
```

Purge old chain data and run:

```bash
make restart
```

Update Cargo:

```bash
make update
```

Update Submodules:

```bash
make update-submodules
```

Update ORML:

```bash
cd orml && git checkout master && git pull
git add orml
cargo update check-all
```

Update Predeploy-Contracts:

```bash
cd highway/predeploy-contracts && git checkout master && git pull
git add predeploy-contracts
cargo update check-all
```

__Note:__ All build command from Makefile are designed for local development purposes and hence have `SKIP_WASM_BUILD` enabled to speed up build time and use `--execution native` to only run using native execution mode.

## 4. Nodes

For Docs on running nodes, check [./docs/nodes.md](./docs/nodes.md)

### 4.1 Embedded docs

Once the project has been built, the following command can be used to explore all parameters and subcommands:

```bash
./target/release/setheum-node -h
```

### 4.2 Release builds

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

### 4.3 On-Chain upgrade builds

Build the wasm runtime with:

```bash
make wasm
```

## 5. EVM - Generate Tokens & Predeploy Contracts

```bash
make generate-tokens
```

__Note:__ All build commands with `SKIP_WASM_BUILD` are designed for local development purposes and hence have the `SKIP_WASM_BUILD` enabled to speed up build time and use `--execution native` to only run use native execution mode.

## 6. Benchmark

### 6.1. Run Benchmark Tests

Run runtime benchmark tests:

```bash
make bench
```

Run module benchmark tests:

```bash
cargo test -p module-poc --all-features
```

### 6.2. Generate Runtime Module Weights Locally

```bash
make benchmark
```

### 6.3. Generate Module weights

Run the module benchmarks and generate the weights file:

```bash
./target/release/setheum-node benchmark \
    --chain=dev \
    --steps=50 \
    --repeat=20 \
    --pallet=module_currencies \
    --extrinsic='*'  \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output=./modules/currencies/src/weights.rs
```

### 6.4. Bench Bot

Bench bot can take care of syncing branch with `master` and generating WeightInfos for module or runtime.

#### 6.4.1. Generate Module Weights

Comment on a PR `/bench module <module_name>` i.e.: `/bench module module_prices`

Bench bot will do the benchmarking, generate `weights.rs` file push and changes into your branch.

#### 6.4.2. Generate Runtime Weights

Comment on a PR `/bench runtime module <module_name>` i.e.: `/bench runtime module module_prices`

Bench bot will do the benchmarking, generate `weights.rs` file and push changes into your branch.

## 5. Fork Setheum Chain

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

## 6. Contributing & Code of Conduct

If you would like to contribute, please fork the repository, introduce your changes and submit a pull request. All pull requests are warmly welcome.

In every interaction and contribution, this
project adheres to the [Contributor Covenant Code of Conduct](./CODE_OF_CONDUCT.md).

## 7. Licenses

The code in this repository is licensed as follows:

1. [Apache-2.0](.LICENSES/Apache-2.0.md)
2. [GPL3](.LICENSES/GPL3.md)
3. [MIT](.LICENSES/MIT.md)

### 7.1  [Apache-2.0 License](.LICENSES/Apache-2.0.md) / [MIT License](.LICENSES/MIT.md)

All crates under [/highend](./highend) directory are licensed under the terms of the [Apache-2.0 License](.LICENSES/Apache-2.0.md) and or the [MIT License](.LICENSES/MIT.md).

* Apache-2.0 License
* MIT License

Unless you explicitly state otherwise, any contribution that you submit to this directory shall be dual licensed as above (as defined in the [Apache-2.0 License](.LICENSES/Apache-2.0.md) and or the [MIT License](.LICENSES/MIT.md)), without any additional terms or conditions.

### 7.2  [GPL-3 License](.LICENSES/GPL3.md)

All rest are licensed under either of the following licenses, at your discretion.

Unless you explicitly state otherwise, any contribution that you submit to this repo (outside of the [./highend](./highend) directory) shall be licensed as above (as defined in the [GNU GPL-3 Version 3.0 or later](.LICENSES/GPL3.md) WITH Classpath exception 2.0, without any additional terms or conditions.
