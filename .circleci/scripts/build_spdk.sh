#!/bin/bash -ex

cd /tmp
git clone git@github.com:spdk/spdk.git

cd /tmp/spdk
git checkout v18.10
git submodule update --init

echo 'APT::Get::Assume-Yes "true"; APT::Get::force-yes "true";' > /etc/apt/apt.conf.d/99force-yes
sudo apt-get update
sudo ./scripts/pkgdep.sh

sudo apt-get install -y module-init-tools

sudo /tmp/spdk/scripts/setup.sh

if [ ! -f "/usr/local/lib/libdpdk.so" ]; then
    cd /tmp/spdk/dpdk
    sudo make install T=`uname -p`-native-linuxapp-gcc \
	    CONFIG_RTE_BUILD_SHARED_LIB=y DESTDIR=/usr/local
else
    echo "dpdk already built"
fi

if [ ! -f "/usr/local/lib/libspdk.so" ]; then
    cd /tmp/spdk
    ./configure --with-shared --with-dpdk=/usr/local
    sudo make install
else
    echo "spdk already built"
fi
