# Docker Layerの機能を使うことでRustで問題になりがちなコンパイル時間の長さを解消する
# RUN, COPY, ADDなどのコマンドを実行するとこのレイヤーが構築され、差分を検証する
# 変更が頻繁に入るファイルは後ろのコマンドに配置することでローカルキャッシュを使って高速化できる
# cargo-chef を使用して先に依存関係のみをコンパイルし、ファイルは後でコンパイルできるようにする
FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
# ロックファイルから計算する
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# 依存関係のみを先にビルドする
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod

### ビルドステージ ###
# FROM rust:1.66.0 as builder

# WORKDIR /app
# RUN apt update && apt install lld clang -y
# COPY . .
# # sqlx-data.jsonを使用してコンパイルを行う
# ENV SQLX_OFFLINE true
# RUN cargo build --release


# ### 実行環境の構築 ###
# FROM rust:1.66.0-slim as runtime

# WORKDIR /app
# COPY --from=builder /app/target/release/zero2prod zero2prod
# COPY configuration configuration
# # プロダクション用の設定ファイルを指定する
# ENV APP_ENVIRONMENT production
# ENTRYPOINT [ "./zero2prod" ]


### 実行環境の構築 ###
FROM debian:bullseye-slim AS runtime

WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean Up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
# プロダクション用の設定ファイルを指定する
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "./zero2prod" ]
