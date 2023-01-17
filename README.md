بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

# Setheum - Powering Scalable Web3 Solutions

<p align="center">
  <img src="./SetheumLabel.jpg" style="width:1300px" />
</p>

* Decentralized
* Ethical
* Interoperable
* Scalable
* Secure

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

## Introduction

### SETHEUM means "Secure, Evergreen, Truthful, Heterogeneous, Economically Unbiased Market".

Setheum is founded,  initiated and facilitated by Muhammad-Jibril B.A. of Setheum Labs, Slixon Technologies and Setheum Foundation to build, steward and support the development and advancement of Ethical Web3 with the Network, its ecosystem and its community to foster the development and adoption of ethical decentralised finance by  and Web3 in general by building and supporting interoperable Secure, Scalable and Decentralized Web3 Infrastructure.
Setheum also deploys Advanced Incentivization mechanisms and economic models modeled under the Jurisdiction of Islamic Finance.

## Getting Started 

This project contains some configuration files to help get started :hammer_and_wrench:

### Initialisation

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

#### Start a development node

The `make run` command will launch a temporary node and its state will be discarded after you terminate the process.
```bash
make run
```

#### Run a persistent single-node chain

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

#### Run tests

```bash
make test
```

#### Run benchmarks

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
    --output=./chassis/lib-serml/currencies/src/weights.rs
```

#### Run in debugger

```bash
make debug
```

#### Nodes

For Docs on running nodes, check [./docs/nodes.md](./docs/nodes.md)

#### Embedded docs

Once the project has been built, the following command can be used to explore all parameters and subcommands:

```bash
./target/release/setheum-node -h
```

#### Release builds

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

#### On-Chain upgrade builds

Build the wasm runtime with:
```bash
make wasm
```

### Update

#### Update Cargo

```bash
make update
```

#### Update ORML

```bash
cd lib-orml && git checkout master && git pull
git add lib-orml
cargo update check-all
```

#### Update Predeploy-Contracts

```bash
cd chassis/lib-sesl/predeploy-contracts && git checkout master && git pull
git add predeploy-contracts
cargo update check-all
```

#### Generate Tokens & Predeploy Contracts - SetheumEVM (SEVM)

```bash
make generate-tokens
```

__Note:__ All build commands with `SKIP_WASM_BUILD` are designed for local development purposes and hence have the `SKIP_WASM_BUILD` enabled to speed up build time and use `--execution native` to only run use native execution mode.

### Bench Bot

Bench bot can take care of syncing branch with `master` and generating WeightInfos for module or runtime.

#### Generate Module weights

##### Generate Weights on Git with PR

Comment on a PR `/bench runtime module <setheum_module_name>` i.e.: `serp_prices`

Bench bot will do the benchmarking, generate `weights.rs` file push changes into your branch.

##### Generate Runtime Module Weights Locally

```bash
make benchmark
```

#### Fork setheum-chain

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

## LICENSES

The projects included in this repo have different licenses which can be found in their individual directories listed below, some use `GPL3`, some `Apache-2.0` while some use the `Business Source License 1.1 (BUSL-1.1)`.

- [ORML License](./chassis/lib-orml/LICENSE.md): Apache-2.0
- [SEPL Licenses](): A mixture of different licenses for different projects (Apache2.0, GPL3, BUSL1.1)

 Other than the listed above (which are all imported `submodules`), the remaining of the primary codebase in this repo, is under `GPL3`. 

### The License Types

For a look, check [./LICENSES](./LICENSES/README.md)

1. [Apache-2.0](./Apache-2.0.md)
2. [BUSL-1.1](./BUSL-1.1.md)
3. [GPL3](./GPL3.md)
