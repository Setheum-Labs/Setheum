#!/usr/bin/env bash

set -e

VERSION=$(git rev-parse --short HEAD)
NODE_NAME=setheum/newrome-node
BUILD_ARGS="--features with-newrome-runtime --features=with-sevm"

docker build -f scripts/Dockerfile . -t $NODE_NAME:$VERSION --no-cache --build-arg GIT_COMMIT=${VERSION} --build-arg BUILD_ARGS="$BUILD_ARGS"
docker push $NODE_NAME:$VERSION
