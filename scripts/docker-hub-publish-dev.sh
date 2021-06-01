#!/usr/node/env bash

VERSION=$(git rev-parse --short HEAD)

docker build . -t setheum/setheum-node:$VERSION --no-cache
docker push setheum/setheum-node:$VERSION
