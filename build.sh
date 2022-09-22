#!/bin/bash -x
set -o errexit

# This script was written to run on netlify to build a production release starting
# from a bare netlify build image.

# Install yarn (1.22.4 is installed by default on netlify, so we keep that).
rm -rf /opt/buildhome/.yarn
curl --silent -o- -L https://yarnpkg.com/install.sh | bash -s -- --version 1.22.4
ls $HOME/.yarn
export PATH="$PATH:$HOME/.yarn/bin"

# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install cobalt
cargo install cobalt-bin --version 0.16.3

# Build static site with cobalt
cd site-base
mkdir -p _data
echo -n "git_rev: " > _data/metadata.yaml
git describe --always --dirty=-modified >> _data/metadata.yaml
cat  _data/metadata.yaml
rm -rf _site
cobalt build
find _site # debug: what was built for cobalt?
cd ..

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

cd hnb-app

# Let yarn install prereqs
yarn install

# Build production release using yarn, place it in dist/
yarn run build

# debug: what was built for yew?
find dist

cd ..

# Remove debug html page for dev use
rm -r hnb-app/dist/index.html

# Put built yew in cobalt build output dir
cp hnb-app/dist/* site-base/_site/hnb-app/

# Move entire site into `dist`
mkdir -p dist
# Make redirect
echo "/ /hnb-app/" > dist/_redirects
mv site-base/_site/* dist/

# FUTURE: build yew and cobalt in their own output dirs, then put into final dir.
# Don't do that now to keep netlify working.
