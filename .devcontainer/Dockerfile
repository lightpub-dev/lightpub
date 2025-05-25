FROM rust:slim

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

# Install packages needed for development
RUN apt-get update && apt-get install -y \
    pkg-config \
    libgexiv2-dev \
    libssl-dev \
    clang \
    lld \
    dos2unix \
    git \
    curl \
    wget \
    mariadb-client \
    nano \
    && rm -rf /var/lib/apt/lists/*

# Install the tools you specified
RUN cargo install sea-orm-cli

RUN rustup component add rustfmt clippy

# Create a non-root user to use if preferred
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME

# Set up user permissions
RUN mkdir -p /workspace && chown $USER_UID:$USER_GID /workspace

# Switch back to dialog for any ad-hoc use of apt-get
ENV DEBIAN_FRONTEND=dialog
ENV CARGO_HOME=/workspace/.cargo
