#!/bin/bash -x
set -euo pipefail

# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal -y
source $HOME/.cargo/env

# Add wasm target for rust
rustup target add wasm32-unknown-unknown

# Install cargo-binstall
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install trunk
cargo binstall trunk@0.21.14

# Build yew app with trunk
cd hnb-app
trunk build --public-url=/hnb-app/ --release
find dist # debug: what was built?
cd ..

# Install cobalt
cargo binstall cobalt-bin@0.19.6

# Build static site with cobalt
cd site-base
rm -rf _site
cobalt build
find _site # debug: what was built for cobalt?
cd ..

# Put built yew in cobalt build output dir
mv hnb-app/dist site-base/_site/hnb-app

# Move entire site into `dist`
mkdir -p dist
# Make redirect
echo "/ /hnb-app/" > dist/_redirects
mv site-base/_site/* dist/

# FUTURE: build yew and cobalt in their own output dirs, then put into final dir.
# Don't do that now to keep netlify working.
