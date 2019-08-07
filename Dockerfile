FROM circleci/rust:latest

ADD . /project

WORKDIR /project

RUN sh .circleci/images/build_spdk.sh

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y

RUN cargo fetch