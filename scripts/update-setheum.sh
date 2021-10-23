#!/usr/bin/env bash

set -e

# cargo clean
WASM_BUILD_TYPE=release cargo run --features=with-setheum-runtime --features=on-chain-release-build -- build-spec --raw --chain setheum-latest > ./resources/chain_spec_mainnet_raw.json
