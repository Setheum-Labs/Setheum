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

[![Setheum version](https://img.shields.io/badge/Setheum-0.9.81-yellow?logo=Parity%20Substrate)](https://setheum.xyz/)
[![License](https://img.shields.io/github/license/Setheum-Labs/Setheum?color=blue)](https://github.com/Setheum-Labs/Setheum/blob/master/LICENSE.md)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](docs/contributor/CONTRIBUTING.md)

[![Rust](https://github.com/Setheum-Labs/Setheum/actions/workflows/rust.yml/badge.svg)](https://github.com/Setheum-Labs/Setheum/actions/workflows/rust.yml)
[![CodeQL](https://github.com/Setheum-Labs/Setheum/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/Setheum-Labs/Setheum/actions/workflows/github-code-scanning/codeql)

[![Website](https://img.shields.io/badge/web-gray?logo=web)](https://setheum.xyz)
[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2FSetheum)](https://twitter.com/Setheum)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/SetheumNetwork)
[![Medium](https://img.shields.io/badge/Medium-gray?logo=medium)](https://medium.com/setheum-labs)

</div>

> NOTE: SETHEUM means `Salam Ethereum`, it also means `The house of gifts` from the name `Seth/Sheeth` meaning `gift` in hebrew and the name of the Prophet Sheeth/Seth in Islam,  it also stands for `Secure, Evergreen, Truthful, Heterogeneous, Economically Unbiased Market`.

<!-- TOC -->
- [Setheum - Powering The New Internet](#setheum---powering-the-new-internet)
  - [1.0. Introduction](#10-introduction)
    - [1.1. Setheum Chain](#11-setheum-chain)
    - [1.2. Ethical DeFi](#12-ethical-defi)
      - [1.2.1. Ethical DeFi Projects](#121-ethical-defi-projects)
  - [2.0. Getting Started](#20-getting-started)
    - [2.1. Build](#21-build)
    - [2.2. Run](#22-run)
      - [2.2.1. Start a development node](#221-start-a-development-node)
      - [2.2.2. Run a persistent single-node chain](#222-run-a-persistent-single-node-chain)
  - [3.0. Development](#30-development)
  - [4.0. Nodes](#40-nodes)
    - [4.1. Embedded docs](#41-embedded-docs)
    - [4.2. Release builds](#42-release-builds)
    - [4.3. On-Chain upgrade builds](#43-on-chain-upgrade-builds)
  - [5.0. EVM - Generate Tokens \& Predeploy Contracts](#50-evm---generate-tokens--predeploy-contracts)
  - [6.0. Benchmark](#60-benchmark)
    - [6.1. Run Benchmark Tests](#61-run-benchmark-tests)
    - [6.2. Generate Runtime Module Weights Locally](#62-generate-runtime-module-weights-locally)
    - [6.3. Generate Module weights](#63-generate-module-weights)
    - [6.4. Bench Bot](#64-bench-bot)
      - [6.4.1. Generate Module Weights](#641-generate-module-weights)
      - [6.4.2. Generate Runtime Weights](#642-generate-runtime-weights)
  - [7.0. Fork Setheum Chain](#70-fork-setheum-chain)
  - [8.0. Contributing \& Code of Conduct](#80-contributing--code-of-conduct)
    - [8.1. ToDo List](#81-todo-list)
  - [9.0. License](#90-license)
<!-- /TOC -->

## 1.0. Introduction

### 1.1. Setheum Chain

Founded November 2019,Setheum achieves a high level of equilibrium in the trilemma by leveraging a Directed Acyclic Graph(DAG) to build the blockchain consensus - making it a Blockchain via DAG, achieve instant finality, high throughput and very fast blocktime while preserving network security and having a fairly decentralised network,

Setheum is a light-speed decentralised blockchain network with EVM and WASM smart contracts, built from a mixture of what we have seen and considered to be some of the best solutions in the industry, improving on scalability, security, user experience, ethics,decentralisation and democratisation. Setheum intends to be the most complete blockchain network in the world. The AlephBFT Consensus Engine powers the Setheum Chain to have near instant finality, high throughput and high scalability and high security.

### 1.2. Ethical DeFi

Ethical DeFi Suite is the DeFi powerhouse of the Setheum Network, providing all kinds of top notch DeFi protocols including an AMM DEX (inspired by Uniswap v3), Decentralised Liquid Staking and Ethical Zero-interest Halal sStablecoins that gives us the properties of both Fiat and Crypto with SlickUSD (USSD) and the Setter (SETR) using an Ethical Collateralized Debt Position (ECDP) mechanism that is over-Collateralized and multi-Collateralised and stable without compromising decentralisation or economic stability, offering stable cryptocurrencies that have scalable value and reliability, setheum provides just that, backed by crypto assets on an efficient zero-interest debt-based system.

#### 1.2.1. Ethical DeFi Projects:

- `Edfis`: DEX (Decentralized Exchange)
  - `Edfis Exchange`: AMM (Automated Market Maker) DEX Protocol inspired by Uniswap v3 design
  - `Edfis Launchpad`: Launchpad Crowdsales protocol for bootstrapping pools on Edfis Exchange
  - `Edfis Launchpool`: Launchpool protocol for bootstrapping pools on Edfis Exchange
- `Moya`: Liquid Staking Protocol
- `Setter`: Unpegged ECDP Stablecoin
- `SlickUSD`: USD Pegged ECDP Stablecoin

## 2.0. Getting Started

This project contains some configuration files to help get started :hammer_and_wrench:

### 2.1. Build

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

Make sure you have `submodule.recurse` set to true to ease submodule use.

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

#### 2.2.1. Start a development node

The `make run` command will launch a temporary node and its state will be discarded after you terminate the process.

```bash
make run
```

#### 2.2.2. Run a persistent single-node chain

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

## 3.0. Development

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

Update ORML:

```bash
cd orml && git checkout master && git pull
git add orml
cargo update check-all
```

__Note:__ All build command from Makefile are designed for local development purposes and hence have `SKIP_WASM_BUILD` enabled to speed up build time and use `--execution native` to only run using native execution mode.

## 4.0. Nodes

For Docs on running nodes, check [./docs/nodes.md](./docs/nodes.md)

### 4.1. Embedded docs

Once the project has been built, the following command can be used to explore all parameters and subcommands:

```bash
./target/release/setheum-node -h
```

### 4.2. Release builds

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

### 4.3. On-Chain upgrade builds

Build the wasm runtime with:

```bash
make wasm
```

## 5.0. EVM - Generate Tokens & Predeploy Contracts

```bash
make generate-tokens
```

__Note:__ All build commands with `SKIP_WASM_BUILD` are designed for local development purposes and hence have the `SKIP_WASM_BUILD` enabled to speed up build time and use `--execution native` to only run use native execution mode.

## 6.0. Benchmark

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
    --output=./blockchain/modules/currencies/src/weights.rs
```

### 6.4. Bench Bot

Bench bot can take care of syncing branch with `master` and generating WeightInfos for module or runtime.

#### 6.4.1. Generate Module Weights

Comment on a PR `/bench module <module_name>` i.e.: `/bench module module_prices`

Bench bot will do the benchmarking, generate `weights.rs` file push and changes into your branch.

#### 6.4.2. Generate Runtime Weights

Comment on a PR `/bench runtime module <module_name>` i.e.: `/bench runtime module module_prices`

Bench bot will do the benchmarking, generate `weights.rs` file and push changes into your branch.

## 7.0. Fork Setheum Chain

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

## 8.0. Contributing & Code of Conduct

If you would like to contribute, please fork the repository, introduce your changes and submit a pull request. All pull requests are warmly welcome.

In every interaction and contribution, this
project adheres to the [Contributor Covenant Code of Conduct](./CODE_OF_CONDUCT.md).

### 8.1. ToDo List

Note> Before adding/removing a TODO, please carefully read the [TODO.md file](./docs/TODO.md)

Whenever you write a TODO in any file, please add a reference to it [here](./docs/TODO.md).
Whenever you remove a TODO in any file, please remove its reference from [here](./docs/TODO.md).

## 9.0. License

The code in this repository is licensed under the [GNU GPL Version 3 License](./LICENSE.md)

Unless you explicitly state otherwise, any contribution that you submit to this repo shall be licensed as above (as defined in the [GNU GPL-3 Version 3.0 or later WITH Classpath-exception-2.0](./LICENSE.md)), without any additional terms or conditions.
