#!/usr/node/env bash

VERSION=$1

if [[ -z "$1" ]] ; then
    echo "Usage: ./scripts/docker-hub-publish.sh VERSION"
    exit 1
fi

docker build . -t setheum/setheum-node:$1 -t setheum/setheum-node:latest
docker push setheum/setheum-node:$1
docker push setheum/setheum-node:latest
