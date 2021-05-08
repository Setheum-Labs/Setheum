#!/usr/bin/env bash

set -e

VERSION=$(git rev-parse --short HEAD)

docker build -f scripts/Dockerfile-parachain . -t setheum/setheum-node:pc-$VERSION --no-cache --build-arg GIT_COMMIT=${VERSION}
docker push setheum/setheum-node:pc-$VERSION
