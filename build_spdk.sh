#!/bin/bash

cd /tmp
git clone https://github.com/spdk/spdk.git

cd /tmp/spdk
git checkout v18.07.1
git submodule update --init

sudo echo 'APT::Get::Assume-Yes "true"; APT::Get::force-yes "true";' > /etc/apt/apt.conf.d/99force-yes
sudo apt-get update
sudo ./scripts/pkgdep.sh

sudo apt-get install -y module-init-tools

sudo /tmp/spdk/scripts/setup.sh

if [ ! -f "/usr/local/lib/libspdk.so" ]; then
    ./configure
    sudo make install
else
    echo "spdk already built"
fi

sudo rm -rf /tmp/spdk
#sudo apt-get remove --purge -y --allow-remove-essential $BUILD_PACKAGES $(apt-mark showauto) && rm -rf /var/lib/apt/lists/*
