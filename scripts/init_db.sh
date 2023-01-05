#!/usr/bin/env bash
set -x
set -eo pipefail

# シェルスクリプトでは処理の実行に必要となるコマンドの依存関係を解決したりはできない
# そのため使用するコマンドがインストールされているかどうかを自前でチェックする
if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 " cargo install --version='~0.6' sqlx-cli \
--no-default-features --features rustls,postgres"
    echo >&2 "to install it."
    exit 1
fi

# DBのユーザーを設定する： デフォルトは 'postgres'
DB_USER="${POSTGRES_USER:=postgres}"
# DBのパスワードを設定する: デフォルトは 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# データベース名を設定する: デフォルトは 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"
# ポート番号を設定する: デフォルトは '5432'
DB_PORT="${POSTGRES_PORT:=5432}"
# ホストを設定する: デフォルトは 'localhost'
DB_HOST="${POSTGRES_HOST:=localhost}"

# データベースが既に起動中の場合には、コンテナを起動しないようにする
# SKIP_DOCKER=true ./scripts/init_db.sh
if [[ -z "${SKIP_DOCKER}" ]]; then
    docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -d postgres \
        postgres -N 1000
    # テスト目的のために最大接続数を増やしておく
fi

# Postgresのインスタンスが起動できた後でデータベースを作成するためのコマンドを実行するため
# データベースの接続確認ができるまで待機する
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    echo >&2 "Postgres is still unavailable - sleeping"
    sleep 1
done

echo >&2 "Postgres is up and running on port ${DB_PORT}!"

# sqlx database create では環境変数に接続文字列 DATABASE_URL が設定されていることを期待する
DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

echo >&2 "Postgres has been migrated, ready to go!"
