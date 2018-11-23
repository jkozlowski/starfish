#!/bin/bash
source $HOME/.cargo/env
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/usr/local/lib"
export RUST_BACKTRACE=1
cargo test $@