FROM rust:bullseye

RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    git \
    make \
    clang \
    redis-tools \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /home
