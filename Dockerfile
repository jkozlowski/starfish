FROM fnichol/rust:nightly

RUN apt-get -q -y update && \
    apt-get -q -y install libaio-dev \
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
    apt-get -q -y clean && \
    apt-get -q -y clean all && \
    rm -rf \
        /var/lib/apt/lists/* \
        /tmp/* \
        /var/tmp/*

EXPOSE 8080
ENTRYPOINT bash