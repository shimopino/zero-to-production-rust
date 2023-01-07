### ビルドステージ ###
FROM rust:1.66.0 as builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
# sqlx-data.jsonを使用してコンパイルを行う
ENV SQLX_OFFLINE true
RUN cargo build --release


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
