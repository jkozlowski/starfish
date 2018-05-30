FROM rust:1.26.0

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
    build-essential && \
    # spdk
    git clone https://github.com/spdk/spdk.git /tmp/spdk && \
    cd /tmp/spdk && \
    git submodule update --init && \
    ./scripts/pkgdep.sh && \
    ./configure && \
    make install && \
    # cleanup
    apt-get -q -y clean && \
    apt-get -q -y clean all && \
    rm -rf \
    /var/lib/apt/lists/* \
    #/tmp/* \
    /var/tmp/*

EXPOSE 8080
ENTRYPOINT bash