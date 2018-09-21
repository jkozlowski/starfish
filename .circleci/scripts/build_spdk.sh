#!/bin/bash

cd /tmp
git clone git@github.com:spdk/spdk.git

cd /tmp/spdk
git checkout v18.07.1
git submodule update --init

export DEBIAN_FRONTEND=noninteractive
apt-get update
./scripts/pkgdep.sh

./configure
make install