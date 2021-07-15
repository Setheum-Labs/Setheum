#!/usr/bin/env bash

set -e

# cargo clean
WASM_BUILD_TYPE=release cargo run --features=with-neom-runtime --features=on-chain-release-build -- build-spec --raw --chain neom-latest > ./resources/neom-dist.json
