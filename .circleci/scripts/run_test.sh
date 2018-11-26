#!/bin/bash

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y
source $HOME/.cargo/env
dd if=/dev/zero of=/tmp/aiofile bs=2048 count=5000
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/usr/local/lib"
cd /home/circleci/project
# LD_PRELOAD workaround for missing shlib dependency produced by rustc
cargo test --all --no-run
LD_PRELOAD=/usr/local/lib/librte_mempool_ring.so.1.1 cargo test --all
