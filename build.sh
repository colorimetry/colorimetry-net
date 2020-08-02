#!/bin/bash -x
set -o errexit

# This script was written to run on netlify to build a production release starting
# from a bare netlify build image. This image seems to have yarn installed but not
# rust.

# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build production release using yarn
yarn run build
