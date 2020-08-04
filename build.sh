#!/bin/bash -x
set -o errexit

# This script was written to run on netlify to build a production release starting
# from a bare netlify build image. This image seems to have yarn installed but not
# rust.

# Install webpack
npm install --global webpack-cli webpack

# Install yarn (1.22.4 is installed by default on netlify, so we keep that).
rm -rf /opt/buildhome/.yarn
curl --silent -o- -L https://yarnpkg.com/install.sh | bash -s -- --version 1.22.4
ls $HOME/.yarn
export PATH="$PATH:$HOME/.yarn/bin"

# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install cobalt
curl --silent -L -o /tmp/cobalt.tar.gz https://github.com/cobalt-org/cobalt.rs/releases/download/v0.16.3/cobalt-v0.16.3-x86_64-unknown-linux-gnu.tar.gz
tar xzf /tmp/cobalt.tar.gz
mv cobalt $HOME/.cargo/bin/cobalt

# Build static site with cobalt
cd site-base
rm -rf _site
cobalt build
find _site # debug: what was built for cobalt?
cd ..

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

cd hnb-app

# Build production release using yarn, place it in dist/
yarn run build

# debug: what was built for yew?
find dist

cd ..

# Put built yew in cobalt build output dir
cp hnb-app/dist/* site-base/_site/hnb-app/

# For now, move entire site into `dist` so netlify finds it again
mkdir -p dist
mv site-base/_site/* dist/

# FUTURE: build yew and cobalt in their own output dirs, then put into final dir.
# Don't do that now to keep netlify working.
