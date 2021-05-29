#!/usr/bin/env bash

set -e

# cargo clean
WASM_BUILD_TYPE=release cargo run --manifest-path bin/setheum/Cargo.toml -- build-spec --chain newrome-latest --raw > ./resources/newrome-pc-dist.json
