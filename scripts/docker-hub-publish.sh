#!/usr/bin/env bash

set -e

VERSION=$1

if [[ -z "$1" ]] ; then
    echo "Usage: ./scripts/docker-hub-publish.sh VERSION"
    exit 1
fi

docker build -f scripts/Dockerfile-dev . -t setheum/setheum-node:$1 -t setheum/setheum-node:latest --build-arg GIT_COMMIT=${VERSION}
docker push setheum/setheum-node:$1
docker push setheum/setheum-node:latest
