#!/bin/bash

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y
source $HOME/.cargo/env
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/usr/local/lib"
cd /home/circleci/project
cargo test --all