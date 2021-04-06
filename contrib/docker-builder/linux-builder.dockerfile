FROM ubuntu:focal as builder
# We label the intermediate images in order to be able to delete them
LABEL stage=builder

ARG DEBIAN_FRONTEND=noninteractive
ARG BPRO_VERSION

RUN apt update && apt upgrade -yqq && apt install -yqq \
    curl \
    libssl-dev \
    libzmq3-dev \
    pkg-config \
    g++ \
    cmake \
    libgtk-3-dev \
    libsqlite3-dev

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

RUN . $HOME/.cargo/env && rustup default nightly

RUN . $HOME/.cargo/env && cargo install bitcoin-pro

# we copy the bin file at root directory because of permissions issue, but there must be a better way
RUN cp /root/.cargo/bin/bitcoin-pro /

WORKDIR /

ENTRYPOINT [ "./bitcoin-pro" ]
