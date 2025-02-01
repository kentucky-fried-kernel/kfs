# Use an official lightweight base image
FROM --platform=linux/amd64 rust:1.70-bullseye

# Install necessary tools
RUN apt-get update && apt-get install -y \
	build-essential \
	curl \
	gcc \
	man \
	gdb \
	lldb \
	git \
	pkg-config \
	nasm \
	mtools \
	xorriso \
	grub-pc-bin \
	grub-efi-amd64-bin \
	qemu \
	qemu-system-x86 \
	zsh \
	&& rm -rf /var/lib/apt/lists/* 


RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" -y

COPY .zshrc /root/.zshrc

RUN rustup target add x86_64-unknown-none
RUN rustup default nightly
RUN rustup component add rust-src
RUN rustup component add rustfmt
RUN rustup component add clippy
RUN cargo install cargo-valgrind

RUN echo 'export PS1="\W> "' >> ~/.bashrc