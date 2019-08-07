FROM circleci/classic:latest

RUN sh .circleci/images/build_spdk.sh

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y

ADD . /project

WORKDIR /project

RUN cargo fetch