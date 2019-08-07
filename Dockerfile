FROM circleci/rust:1.36.0-buster

ADD build_spdk.sh /project/build_spdk.sh

RUN sh /project/build_spdk.sh

#RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y

ADD . /project

RUN cd /project && cargo fetch