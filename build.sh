#!/bin/bash -x
set -o errexit

# This script was written to run on netlify to build a production release starting
# from a bare netlify build image. This image seems to have yarn installed but not
# rust.

# # Install rust
# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# source $HOME/.cargo/env

# # Install cobalt
# curl --silent -L -o /tmp/cobalt.tar.gz https://github.com/cobalt-org/cobalt.rs/releases/download/v0.16.3/cobalt-v0.16.3-x86_64-unknown-linux-gnu.tar.gz
# tar xzf /tmp/cobalt.tar.gz
# mv cobalt $HOME/.cargo/bin/cobalt

# Build static site with cobalt
cd site-base
rm -rf _site
cobalt build
find _site # debug: what was built for cobalt?
cd ..

# # Install wasm-pack
# curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build production release using yarn, place it in dist/
rm -rf dist/*
yarn run build

# debug: what was built for yew?
find dist

# Put built yew in cobalt build output dir
mv dist/* site-base/_site/app/

# For now, move entire site into `dist` so netlify finds it again
mv site-base/_site/* dist/

# FUTURE: build yew and cobalt in their own output dirs, then put into final dir.
# Don't do that now to keep netlify working.
