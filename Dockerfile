FROM rust:1.26.1

RUN apt-get -q -y update && \
    apt-get -q -y install \
    # libaio
    libaio-dev \
    libclang-dev \
    clang \
    pkg-config \
    xfslibs-dev \
    gdb \
    lldb \
    libssl-dev \
    libsnappy-dev \
    automake \
    libtool \
    build-essential \
    vim \
    gdbserver

# spdk
RUN git clone https://github.com/spdk/spdk.git /tmp/spdk && \
    # DPDK debug mode
    export EXTRA_CFLAGS='-O0 -g' && \
    cd /tmp/spdk && \
    git checkout v18.04 && \
    git submodule update --init && \
    ./scripts/pkgdep.sh

RUN cd /tmp/spdk && \
    ./configure --enable-debug && \
    make install && \
    # PCI_WHITELIST="none" ./scripts/setup.sh && \
    cd /tmp/spdk/dpdk && \
    make install

# cleanup
#RUN apt-get -q -y clean && \
#apt-get -q -y clean all && \
#rm -rf \
#/var/lib/apt/lists/* \
#/tmp/* \
#/var/tmp/*

EXPOSE 5801 5801
ENTRYPOINT bash