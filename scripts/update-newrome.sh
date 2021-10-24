#!/usr/bin/env bash

set -e

# cargo clean
WASM_BUILD_TYPE=release cargo run --features with-newrome-runtime --features with-ethereum-compatibility -- build-spec --raw --chain newrome-latest > ./resources/chain_spec_testnet_raw.json
