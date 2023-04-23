# ビルド時に必要な設定を行う
FROM rust:1.69.0 as builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# 実行時に必要な設定を行う
FROM rust:1.69.0-slim as runtime
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT [ "./zero2prod" ]