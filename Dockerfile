FROM ubuntu:eoan-20190717.1

ADD build_spdk.sh /project/build_spdk.sh

RUN sh /project/build_spdk.sh

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y

ADD . /project

ENV PATH=/root/.cargo/bin:$PATH

RUN cd /project && \ 
    git submodule update --init --recursive && \
    cargo fetch && \
    rustup component add rustfmt-preview && \
    rustup component add clippy-preview