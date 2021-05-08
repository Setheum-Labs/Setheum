#!/usr/bin/env bash

set -e

PROJECT_ROOT=`git rev-parse --show-toplevel`

# generate-tokens
cargo test -p setheum-primitives -- --ignored

# generate-predeployed-contracts
cd sevm-predeployed-contracts
yarn
yarn run generate-bytecode
