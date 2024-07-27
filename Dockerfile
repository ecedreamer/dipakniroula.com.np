FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y build-essential pkg-config libssl-dev

ENV USER=root

WORKDIR /project