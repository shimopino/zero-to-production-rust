#!/usr/bin/env bash
set -x
set -eo pipefail

# 利用するコマンドをインストールしているのかどうかを確認する
if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed." exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 " cargo install --version='~0.6' sqlx-cli \
--no-default-features --features rustls,postgres" echo "to install it." >&2
    exit 1
fi

# 各変数に対して環境変数を確認し、なければデフォルト値を設定する
DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

# Allow to skip Docker if a dockerized Postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]; then
    # if a postgres container is running, print instructions to kill it and exit
    RUNNING_POSTGRES_CONTAINER=$(docker ps --filter 'name=postgres' --format '{{.ID}}')
    if [[ -n $RUNNING_POSTGRES_CONTAINER ]]; then
        echo >&2 "there is a postgres container already running, kill it with"
        echo >&2 "    docker kill ${RUNNING_POSTGRES_CONTAINER}"
        exit 1
    fi
    # Launch postgres using Docker
    docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -d \
        --name "postgres_$(date '+%s')" \
        postgres -N 1000
    # ^ Increased maximum number of connections for testing purposes
fi

# PostgreSQLが起動するまで待機する
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    echo >&2 "Postgres is still unavailable - sleeping"
    sleep 1
done
echo >&2 "Postgres is up and running on port ${DB_PORT}!"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run # このコマンドを追加

echo >&2 "Postgres has been migrated, ready to go!"
