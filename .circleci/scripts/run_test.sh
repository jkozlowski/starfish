#!/bin/bash

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y
source $HOME/.cargo/env
dd if=/dev/zero of=/tmp/aiofile bs=2048 count=5000
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/usr/local/lib"
cd /home/circleci/project

# This seems to work
DRIVER_OVERRIDE=vfio-pci HUGEMEM=8192 ./spdk/scripts/setup.sh config

RUST_BACKTRACE=trace cargo test --all -- --nocapture