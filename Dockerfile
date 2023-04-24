# ========== build ==========
# ビルド時に必要な設定を行う
FROM rust:1.69.0 as builder
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# # https://github.com/emk/rust-musl-builder
# FROM ekidd/rust-musl-builder:stable as builder
# WORKDIR /app
# COPY . .
# ENV SQLX_OFFLINE true
# RUN cargo build --release

# ========== runtime ==========
# 実行時に必要な設定を行う
# FROM rust:1.69.0-slim as runtime
# COPY --from=builder /app/target/release/zero2prod zero2prod
# COPY configuration configuration
# ENV APP_ENVIRONMENT=production
# ENTRYPOINT [ "./zero2prod" ]

# さらに軽量にするために debian から構築する
# FROM debian:bullseye-slim as runtime
# WORKDIR /app
# # 必要なライブラリのみをインストールする
# RUN apt update -y \
#     && apt install -y --no-install-recommends openssl ca-certificates \
#     && apt autoremove -y \
#     && apt clean -y \
#     && rm -rf /var/lib/apt/lists/*
# COPY --from=builder /app/target/release/zero2prod zero2prod
# COPY configuration configuration
# ENV APP_ENVIRONMENT=production
# ENTRYPOINT [ "./zero2prod" ]

# https://github.com/GoogleContainerTools/distroless/blob/main/examples/rust/Dockerfile
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT [ "./zero2prod" ]
