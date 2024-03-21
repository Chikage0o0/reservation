FROM mcr.microsoft.com/devcontainers/base:alpine

# Add lld 、protobuf-dev 、protoc
RUN apk add --no-cache lld protobuf-dev pipx pgcli

USER vscode

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Install cargo-binstall and cargo tools
RUN . $HOME/.cargo/env && curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash && cargo binstall cargo-deny cargo-nextest cargo-watch sqlx-cli --no-confirm

RUN pipx install pre-commit
