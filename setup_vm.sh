#!/bin/bash

sudo apt-get update
mkdir ~/code
cd code
git clone git@github.com:jkozlowski/starfish.git
curl https://sh.rustup.rs -sSf | sh
cargo install bindgen

cd /tmp
git clone git@github.com:spdk/spdk.git

cd /tmp/spdk
git checkout v18.04
git submodule update --init
# if [ -s /etc/redhat-release ] || [ -s /etc/system-release ] ; then
sudo ./scripts/pkgdep.sh

#export EXTRA_CFLAGS='-fPIC'
#export CFLAGS='-fPIC'
#export CXXFLAGS='-fPIC'

# !!!!!!!!! Need to manually make sure that -fPIC ends up in dpdk build; edit dpdkbuild/Makefile !!!!!!!
# Otherwise going to get errors

./configure
sudo make install

sudo apt-get install nvme-cli
# sudo yum install nvme-cli

sudo HUGEMEM=1024 PCI_WHITELIST="vfio" ./scripts/setup.sh

# Need to run dpdk applications as root :(
 