#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

rustup default nightly-2021-05-09

rustup target add wasm32-unknown-unknown --toolchain nightly-2021-05-09
