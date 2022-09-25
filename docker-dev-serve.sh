#!/bin/bash
set -o errexit

# While the Dockerfile builds the static website for production, we can also use
# docker for development in a reproducible, isolated environment.

apt-get update
apt-get install -y \
    libssl-dev \
    pkg-config

cd hnb-app
yarn run start:dev
