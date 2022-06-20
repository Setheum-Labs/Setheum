#!/usr/bin/env bash

set -e

PROJECT_ROOT=`git rev-parse --show-toplevel`

# generate-tokens
cargo test -p system-primitives -- --ignored

# generate-predeploy-contracts
cd lib-serml/sevm/predeploy-contracts
yarn
yarn run generate-bytecode
