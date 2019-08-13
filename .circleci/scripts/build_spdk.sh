#!/bin/bash

git submodule update --init

echo 'APT::Get::Assume-Yes "true"; APT::Get::force-yes "true";' > /etc/apt/apt.conf.d/99force-yes

cd spdk-sys/spdk
sudo apt-get update
sudo ./scripts/pkgdep.sh

sudo apt-get install -y module-init-tools

sudo ./scripts/setup.sh

if [ ! -f "/usr/local/lib/libspdk.so" ]; then
    ./configure
    sudo make install
else
    echo "spdk already built"
fi
