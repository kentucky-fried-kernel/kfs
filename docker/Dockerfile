FROM --platform=linux/amd64 rust:1.90-bullseye

RUN apt update && \
    apt install -y \
    qemu-system-x86-64 && \
    build-essential && \
    gcc-multilib && \
    make && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-none && \
    rustup default nightly && \
    rustup component add rust-src && \
    rustup component add clippy && \
    rustup component add rustfmt
