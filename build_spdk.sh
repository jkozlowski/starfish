#!/bin/bash -v

apt-get update
# TODO(jkozlowski) Put in custom docker container
apt-get install -y \
    build-essential \
    git mercurial xvfb apt \
    locales sudo openssh-client ca-certificates tar gzip parallel \
    net-tools netcat unzip zip bzip2 gnupg curl wget make

git clone https://github.com/jkryl/spdk-sys.git

cd /tmp/spdk-sys
git submodule update --init --recursive

# TODO(jkozlowski) Add this to spdk-sys
./spdk-sys/spdk/scripts/pkgdep.sh
# TODO(jkozlowski) What is this for?
apt-get install -y module-init-tools

sh ./spdk-sys/build.sh
cp spdk-sys/build/libspdk_fat.so /usr/local/lib

./spdk-sys/spdk/scripts/setup.sh

cd spdk-sys/spdk
make install
