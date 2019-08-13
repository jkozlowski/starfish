FROM ubuntu:eoan-20190717.1

 ADD build_spdk.sh /project/build_spdk.sh

 RUN sh /project/build_spdk.sh

 RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y

 ADD . /project

 ENV PATH=$HOME/.cargo/bin:$PATH

 RUN cd /project && source $HOME/.cargo/env && cargo fetch