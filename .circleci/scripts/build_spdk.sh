#!/bin/bash

cd /tmp
git clone git@github.com:spdk/spdk.git

cd /tmp/spdk
git checkout v18.07.1
git submodule update --init

echo 'APT::Get::Assume-Yes "true"; APT::Get::force-yes "true";' > /etc/apt/apt.conf.d/99force-yes
apt-get update
./scripts/pkgdep.sh

./configure
make install
