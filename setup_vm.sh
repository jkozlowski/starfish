#!/bin/bash

sudo apt-get update
mkdir ~/code
cd code
git clone git@github.com:jkozlowski/starfish.git
curl https://sh.rustup.rs -sSf | sh

cd /tmp
git clone git@github.com:spdk/spdk.git

cd /tmp/spdk
git checkout v18.04
git submodule update --init
# if [ -s /etc/redhat-release ] || [ -s /etc/system-release ] ; then
sudo ./scripts/pkgdep.sh
# EXPORT EXTRA_CFLAGS='-O0 -g'
# Fix failures with 'can not be used when making a shared object; recompile with -fPIC'
# export EXTRA_CFLAGS='-fno-pie -fPIC'
# export CFLAGS='-fno-pie -fPIC'
# export CXXFLAGS='-fno-pie -fPIC'

export EXTRA_CFLAGS='-fPIC'
export CFLAGS='-fPIC'
export CXXFLAGS='-fPIC'
./configure
# add CONFIG_RTE_BUILD_SHARED_LIB?=y to CONFIG.local
# !!!!!!!!! Need to manually make sure that -fPIC ends up in dpdk build; edit dpdkbuild/Makefile !!!!!!!
# Otherwise going to get errors
# ./configure --enable-debug
sudo make install

cd /tmp/spdk/dpdk
sudo make install

sudo apt-get install nvme-cli
# sudo yum install nvme-cli

sudo HUGEMEM=1024 PCI_WHITELIST="vfio" ./scripts/setup.sh

# Need to run dpdk applications as root :(

sudo wget -O /usr/local/bin/rmate 