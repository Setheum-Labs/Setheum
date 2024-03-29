#!/usr/bin/env bash

set -e

PROJECT_ROOT=`git rev-parse --show-toplevel`

# generate-tokens
cargo test -p setheum-primitives -- --ignored

# generate-predeploy-contracts
cd blockchain/submodules/predeploy-contracts
yarn
yarn run generate-bytecode
