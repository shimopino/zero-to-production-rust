# コンテナ化

環境起因による失敗の可能性を減らすために Docker を使用したアプリケーションの仮想化を行う

まずは `Dockerfile` を以下のようにシンプルな構成にしてビルド生成物を起動するというスタイルを採用する

```dockerfile
FROM rust:1.68.0
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
RUN cargo build --release
ENTRYPOINT [ "./target/release/zero2prod" ]
```

このビルド処理を実行するとコンパイル時に `sqlx` のコンパイルエラーが発生する

```bash
error: error communicating with database: Cannot assign requested address (os error 99)
  --> src/routes/subscriptions.rs:19:11
   |
19 |       match sqlx::query!(
   |  ___________^
20 | |         r#"
21 | |         INSERT INTO subscriptions (id, email, name, subscribed_at)
22 | |         VALUES ($1, $2, $3, $4)
...  |
27 | |         Utc::now()
28 | |     )
   | |_____^
   |
   = note: this error originates in the macro `$crate::sqlx_macros::expand_query` which comes from the expansion of the macro `sqlx::query` (in Nightly builds, run with -Z macro-backtrace for more info)
```

Github Actions のステップを定義する際にも発生したが、 `sqlx` はコンパイル時に型安全性を担保する代わりに、コンパイル時に実際のデータベースにアクセスを行い、テーブルに対してクエリを検証できる必要がある

Github Actions の時にはサービスコンテナを利用して、同じ Docker ネットワーク上にパイプラインを実行しているサーバーと DB サーバーを配置することで、指定されている環境変数を利用して DB サーバーにアクセスすることで問題の回避を行なった

今回は Docker ビルドに起因する課題なので、同じやり方ではなく `sqlx` がサポートしているオフライン機能を利用する

```bash
[dependencies.sqlx]
version = "^0.6"
default-features = false
features = [
    # ...
    # DBサーバーに接続しない状態でもコンパイル可能にする機能
    "offline"
]
```

以下のようにコマンドを実行すれば `sqlx-data.json` というファイルが生成され、DB サーバーに接続する必要なくコンパイルが実行できるようになる

```bash
cargo sqlx prepare -- --lib
```

- [offline mode](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query)

`sqlx` は `DATABASE_URL` 環境変数が存在する場合には実際にその変数を使用してデータベースに対してビルドするが、偶然この環境変数が登録されていても問題ないように `SQLX_OFFLINE` 環境変数を設定するだけでオフライン前提でのビルドを実行することが可能となる

- [force building in offline mode](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#force-building-in-offline-mode)

これは最新の `sqlx-data.json` をバージョン管理する必要があるが、以下のように検証処理を CI に組み込んでおけば、コミット漏れも防ぐことが可能である

```bash
cargo sqlx prepare --check
```
