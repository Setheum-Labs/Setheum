#!/usr/bin/env bash

# cd to the root of the repo
cd "$(git rev-parse --show-toplevel)"

cp README.md contracts/
cp -r build contracts/build
