FROM --platform=linux/amd64 rust:1.90-bullseye

RUN apt update && \
	apt install -y \
	qemu-system-x86-64 && \
	rm -rf /var/lib/apt/lists/*

RUN rustup default nightly && \
	rustup update nightly && \
	rustup target add i386-unknown-none && \
	rustup component add rust-src && \
	rustup component add clippy && \
	rustup component add rustfmt
