#!/usr/bin/env bash

set -e

# cargo clean
WASM_BUILD_TYPE=release cargo run --manifest-path node/setheum/Cargo.toml --features=on-chain-release-build -- build-spec --raw --chain neom-latest > ./resources/neom-dist.json
