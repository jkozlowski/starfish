#!/bin/bash -ve

apt-get update
# TODO(jkozlowski) Put in custom docker container
apt-get install -y \
    build-essential \
    git mercurial xvfb apt \
    locales sudo openssh-client ca-certificates tar gzip parallel \
    net-tools netcat unzip zip bzip2 gnupg curl wget make \
    apt-utils kmod 

git clone https://github.com/jkryl/spdk-sys.git /tmp/spdk-sys

cd /tmp/spdk-sys
git submodule update --init --recursive

# TODO(jkozlowski) Add this to spdk-sys
./spdk/scripts/pkgdep.sh

sh ./build.sh --with-igb-uio-driver
cp build/libspdk_fat.so /usr/local/lib

cd spdk
make install
