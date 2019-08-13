#!/bin/bash

git submodule update --init

sudo echo 'APT::Get::Assume-Yes "true"; APT::Get::force-yes "true";' > /etc/apt/apt.conf.d/99force-yes

sudo apt-get update
# TODO(jkozlowski) Add this to spdk-sys
sudo ./spdk-sys/spdk/scripts/pkgdep.sh
# TODO(jkozlowski) What is this for?
sudo apt-get install -y module-init-tools

sh ./spdk-sys/build.sh

if [ ! -f "/usr/local/lib/libspdk.so" ]; then
    cd spdk-sys/spdk
    sudo make install
else
    echo "spdk already built"
fi
