FROM --platform=linux/amd64 rust:1.89-bullseye

ARG USER="thegoat"

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

COPY .zshrc /home/$USER/.zshrc

ARG USER_UID="1000"
ARG USER_GID="1000"

RUN apt-get update && apt-get install -y sudo zsh && apt-get clean

RUN groupadd --gid $USER_GID $USER && \
    useradd --uid $USER_UID --gid $USER_GID -m $USER -s /bin/zsh && \
    chown -R $USER_UID:$USER_GID /home/$USER/ && \
    echo $USER ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USER && \
    chmod 0440 /etc/sudoers.d/$USER

RUN rustup target add x86_64-unknown-none && \
    rustup default nightly && \
    rustup component add rust-src && \
    rustup component add rustfmt && \
    rustup component add clippy && \
    cargo install cargo-valgrind && \
    chown -R $USER_UID:$USER_GID /usr/local/cargo /usr/local/rustup

USER $USER

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" -y

COPY --chown=$USER_UID:$USER_GID .zshrc /home/$USER/
