#!/bin/bash

sudo apt-get update
mkdir ~/code
cd code
git clone git@github.com:jkozlowski/starfish.git
curl https://sh.rustup.rs -sSf | sh
cargo install bindgen
rustup component add clippy-preview
rustup component add rustfmt-preview
rustup component add rustfmt-preview --toolchain nightly

cd /tmp
git clone git@github.com:spdk/spdk.git

cd /tmp/spdk
git checkout v18.07.1
git submodule update --init
sudo ./scripts/pkgdep.sh

./configure
sudo make install
./scripts/setup.sh

# Need to run dpdk applications as root :(
 