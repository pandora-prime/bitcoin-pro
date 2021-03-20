FROM ubuntu:focal as builder

ARG DEBIAN_FRONTEND=noninteractive
ARG BPRO_VERSION

RUN apt update && apt upgrade -yqq && apt install -yqq \
    curl \
    cargo \
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

FROM alpine:latest

WORKDIR /
COPY --from=builder /root/.cargo/bin/bitcoin-pro .

# TODO: enable running the bin directly inside a container 
# ENTRYPOINT [ "bitcoin-pro" ]
