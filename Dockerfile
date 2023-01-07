FROM rust:1.66.0

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
# sqlx-data.jsonを使用してコンパイルを行う
ENV SQLX_OFFLINE true
RUN cargo build --release
# プロダクション用の設定ファイルを指定する
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "./target/release/zero2prod" ]