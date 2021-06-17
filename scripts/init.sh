#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

if [ -z $CI ] ; then
   rustup update nightly-2021-03-15
fi

rustup target add wasm32-unknown-unknown --toolchain nightly-2021-03-15
rustup default nightly-2021-03-15
