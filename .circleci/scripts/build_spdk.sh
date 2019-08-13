#!/bin/bash -v

git submodule update --init --recursive

sudo apt-get update
# TODO(jkozlowski) Put in custom docker container
sudo apt-get install -y \
    git mercurial xvfb apt \
    locales sudo openssh-client ca-certificates tar gzip parallel \
    net-tools netcat unzip zip bzip2 gnupg curl wget make
# TODO(jkozlowski) Add this to spdk-sys
sudo ./spdk-sys/spdk/scripts/pkgdep.sh
# TODO(jkozlowski) What is this for?
sudo apt-get install -y module-init-tools

sh ./spdk-sys/build.sh

sudo ./spdk-sys/spdk/scripts/setup.sh

if [ ! -f "/usr/local/lib/libspdk.so" ]; then
    cd spdk-sys/spdk
    sudo make install
else
    echo "spdk already built"
fi
