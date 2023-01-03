# Zero to Production in Rust

[Zero To Production In Rust](https://www.zero2prod.com/index.html?country=Japan&discount_code=VAT20) のコード写経する

- [Zero to Production in Rust](#zero-to-production-in-rust)
  - [環境設定](#環境設定)
  - [API の仕様](#api-の仕様)
  - [CI の検証](#ci-の検証)
  - [基礎](#基礎)
  - [Rust でのテストのやり方](#rust-でのテストのやり方)

## 環境設定

本リポジトリでの環境設定は下記の状態である

```bash
>> rustc --version
rustc 1.66.0 (69f9c33d7 2022-12-12)

>> cargo --version
cargo 1.66.0 (d65d197ad 2022-11-15)
```

本書では IDE として「IntelliJ Rust」を推奨しているが、本リポジトリでは VSCode と rust-analyzer の拡張機能を使用する

ソースコードが変更されるたびにコンパイルなどを実行するために `cargo-watch` を使用する

```bash
# https://crates.io/crates/cargo-watch
cargo install cargo-watch

# ソースコードが変更された時にどのコマンドを順番に実行するのか指定する
cargo watch -x check -x test -x run
```

テストコードのコードカバレッジを計算するために `cargo-tarpaulin` を使用する

```bash
# https://crates.io/crates/cargo-tarpaulin
cargo install cargo-tarpaulin

# テストコードを無視して、アプリケーションコードのみのカバレッジを計算する
cargo tarpaulin --ignore-tests
```

クレートの脆弱性を検査するために `cargo-audit` を使用する

```bash
# https://crates.io/crates/cargo-audit
cargo install cargo-audit

# 脆弱性を検査する
cargo audit
```

[`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) だと脆弱性の検査に加えて、ライセンスの制限なども含めてさまざまなことを実施できる

## API の仕様

本書では実装する対象をユーザーストーリーという形式で提供している

- ブログの訪問者として、ニュースレターを購読したい、なぜなら、ブログに新しい記事が投稿されたことを知りたいからだ
- ブログの著者として、購読者に対してメールを送信したい、なぜなら、新しい記事を執筆したことを購読者に教えたいからだ

はじめから上記の仕様を満たした上で非機能的な内容も実装するのではなく、最初は仕様をある程度満たすようにサービスを構築していき、徐々に耐障害性やリトライ機能の追加、新規購読者への確認メールなどを追加していく

## CI の検証

CI パイプラインで使用する Github Actions をローカルで検証するために [nektos/act](https://github.com/nektos/act) を使用する

```bash
# nektos/act をインストールする
brew install act

# 実行できるパイプラインの一覧を表示する
act -l

# 実行
# 何も指定しなければ push イベントで実行する
act

# 特定のイベントで実行
act pull_request

# 特定のジョブを実行
act -j test
```

## 基礎

Rust で API を構築するために [actix-web](https://actix.rs/) を使用する

公式が提供しているサンプルコード通りに実装して挙動を確認する

```rs
use actix_web::{web, App, HttpRequest, HttpServer, Responder};

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
```

これでサーバーを起動して HTTP リクエストを送信すれば、レスポンスが返ってくることがわかる

```bash
>> cargo run

>> curl http://127.0.0.1:8000/shimokawa
Hello shimokawa!%
```

このコードが具体的に何をしているのかは、マクロを展開できる `cargo-expand` を使用すればある程度把握することができる

```bash
>> cargo install cargo-expand
>> rustup toolchain install nightly --allow-downgrade
>> cargo +nightly expand
```

これでマクロを展開すると、以下のように `main` 関数から非同期処理の `async` キーワードがなくなり、非同期のランタイムを起動して `Future` が完了することを期待していることがわかる

```rs
fn main() -> std::io::Result<()> {
    <::actix_web::rt::System>::new()
        .block_on(async move {
            {
                HttpServer::new(|| {
                        App::new()
                            .route("/", web::get().to(greet))
                            .route("/{name}", web::get().to(greet))
                    })
                    .bind("127.0.0.1:8000")?
                    .run()
                    .await
            }
        })
}
```

## Rust でのテストのやり方

Rust でテストを実施する時には下記の選択肢が存在している

1. 本番コードのモジュールにテストコードを組み込む
2. 外部の `tests` フォルダーにテストコードを配置する
3. パブリックなドキュメントの中にテストコードを組み込む（`doc tests`）

最初の方法の特徴は本番コードと同じモジュールに配置することで、コードに特権的にアクセスすることができ、公開されていない API にもアクセスしてテストを実行することが可能となる

残りの方法は、どちらの場合でも別のバイナリにコンパイルされ、コードへのアクセスに対しても、依存関係としてクレートを追加した時の同じアクセス権限を有することになり、結合テストを実施形式となる

`main.rs` でサーバーを起動するコードを含めている場合、その部分までバイナリにコンパイルされてしまうため、結合テストコード側からアクセスすることができないため、まずはファイルをバイナリとライブラリに分解する必要がある

そのために以下のようにプロジェクトの構成を変更する

```toml
[lib]
# どのようなパスでも指定することができる
# nameフィールドを使ってライブラリ名を指定すると、指定しない場合は `package.name` を使用する
path = "src/lib.rs"

# この記法はTOMLファイルでの配列を意味する
[[bin]]
# 1つのプロジェクトにつき1つのライブラリを持っているが、バイナリは複数指定することができる
# もしも複数のライブラリを管理したい場合にはWorkspace機能を使用する
path = "src/main.rs"
name = "zero2prod"
```
