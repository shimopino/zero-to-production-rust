# 必要なライブラリ一式をインストール
apt update &&
    apt install -y \
        jq \
        sudo \
        zsh \
        vim &&
    apt clean &&
    rm -rf /var/lib/apt/list/*

# Install Rust
rustup component add rustfmt
rustup component add clippy
cargo install cargo-expand
cargo install cargo-edit
cargo install cargo-watch

# Setup oh-my-zsh
