FROM --platform=linux/amd64 rust:1.90-bullseye

RUN apt update && \
    apt install -y \
    build-essential \
    gcc-multilib \
    qemu-system-x86 \
    qemu \
    qemu-utils \
    grub-common \
    grub-pc-bin \
    grub-efi-amd64-bin \
    xorriso \
    mtools && \
    rm -rf /var/lib/apt/lists/*

# We cannot use 'ld' directly in the build process, since MacOS has a different
# version of 'ld' and needs to install a port called 'i386-elf-ld'.
RUN cp $(which ld) /usr/bin/i386-elf-ld && \
    cp $(which as) /usr/bin/i386-elf-as

RUN rustup target add i386-unknown-none && \
    rustup default nightly && \
    rustup component add rust-src && \
    rustup component add clippy && \
    rustup component add rustfmt
