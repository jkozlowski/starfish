FROM circleci/rust:latest

ADD . /project

RUN sh /project/.circleci/images/build_spdk.sh

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y

RUN cd /project && cargo fetch