#!/usr/bin/env bash

set -e

# cargo clean
WASM_BUILD_TYPE=release cargo run --manifest-path bin/setheum-dev/Cargo.toml --features with-ethereum-compatibility -- build-spec --raw --chain newrome-latest > ./resources/newrome-dist.json
