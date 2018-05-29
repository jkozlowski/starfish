FROM rust:1.26.0

COPY smp-spdk/spdk/scripts/pkgdep.sh /tmp/pkgdep.sh

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
    ./tmp/pkgdep.sh  && \
    # cleanup
    apt-get -q -y clean && \
    apt-get -q -y clean all && \
    rm -rf \
    /var/lib/apt/lists/* \
    /tmp/* \
    /var/tmp/*

EXPOSE 8080
ENTRYPOINT bash