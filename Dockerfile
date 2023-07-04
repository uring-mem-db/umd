FROM rust:bullseye

RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    git \
    make \
    clang \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt

WORKDIR /home
