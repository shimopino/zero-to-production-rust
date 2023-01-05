# Zero to Production in Rust

[Zero To Production In Rust](https://www.zero2prod.com/index.html?country=Japan&discount_code=VAT20) のコード写経する

- [Zero to Production in Rust](#zero-to-production-in-rust)
  - [環境設定](#環境設定)
  - [API の仕様](#api-の仕様)
  - [CI の検証](#ci-の検証)
  - [基礎](#基礎)
  - [Rust でのテストのやり方](#rust-でのテストのやり方)
  - [テストで仕様を表現する](#テストで仕様を表現する)
  - [データベースの操作](#データベースの操作)
  - [不確実な状況への対応](#不確実な状況への対応)
  - [ログの一意性](#ログの一意性)
  - [ログからトレースへ](#ログからトレースへ)

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

## テストで仕様を表現する

ニュースレターの購読を行う際に、ユーザーに有効であり、識別子となる名前とメールアドレスのペアを入力してもらい、どちらかが欠けていれば `Bad Request` として返却することを考える

リクエストボディに対して下記のようにパラメータを設定する（今回は JSON 形式ではないの、そのまま文字列形式のまま送信している）

```rs
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let address = spwan_app();
    let client = reqwest::Client::new();

    // Act
    // パーセントエンコーディングのため、空白は %20 でエンコードされている
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
}
```

## データベースの操作

この書籍では Rust のエコシステムのうち、下記の 3 つの観点からどのクレートを使用するのか選定している

- コンパイル時の安全性
- SQL first なのか DSL first なのか
- async なのか async のインターフェースなのか

本書では `sqlx` を使用する

これはコンパイル時に型エラーなどを検知して失敗させ、SQL 駆動でクエリを発行し、非同期インターフェースをサポートしている

データベースを使用することで副作用が発生するようになるが、テスト時の検証する方法として複数の公開 API を組み合わせてテストする方法と、直接 SQL を発行してデータベースの状態を検証する方法が存在している

今回はまず SQL クエリを発行する形式で進めていき、データを取得する API が完成した段階でテストをリファクタリングしていく

```bash
>> cargo install --version="~0.6" sqlx-cli --no-default-features \
    --feature rustls,postgres

>> sqlx --help
sqlx-cli 0.6.2
Jesper Axelsson <jesperaxe@gmail.com>, Austin Bonander <austin.bonander@gmail.com>
Command-line utility for SQLx, the Rust SQL toolkit.

USAGE:
    sqlx <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    database    Group of commands for creating and dropping your database
    help        Print this message or the help of the given subcommand(s)
    migrate     Group of commands for creating and running migrations
    prepare     Generate query metadata to support offline compile-time verification
```

マイグレーションを行う場合には下記のように実施する

```bash
>> export DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/newsletter
>> sqlx migrate add create_subscriptions_table
```

これでトップディレクトリに `migrations` というマイグレーションを管理するためのディレクトリが作成され、そこに空のファイルが格納されていることがわかる

マイグレーションファイルの準備ができればあとは実行するのみである

```bash
>> sqlx migrate run
```

## 不確実な状況への対応

結合テストでは自分達で想定したアプリケーションの成功パスと失敗パスをテストしているが、実際にアプリケーションを動かしている環境では、以下のようなどのような結果になるのかわからない状況も存在する

- データベースへの接続が失われた時に、どのような挙動になるのか
- 悪意のあるペイロードを指定して何らかのインジェクションを行おうとするとどうなるのか
- 想定していない量のリクエストが来るとどうなるのか
- アプリケーションが長い間再起動されなかったために発生したメモリリークは何を引き起こすのか

こうした状況は「既知の未知数」と呼ばれ、特徴的なことは実際にアプリケーションを稼働している状況で発生しうるものであり、テストでは再現困難なことである

抗体状況に対応するには、実行中のアプリケーションの情報を自動的に収集し、ある時点でのシステムの状態に関する質問に答えることができるようにしておく必要がある

ここで「ある時点でのシステム状態に関する質問」とは、今回のように未知数の状態の場合には事前に想定することが難しいため、観察可能なアプリケーションを構築することが重要となる

> 観測可能性とは、システムの環境について任意の質問をすることができる状態を指しており、何を質問したいのかを事前に知っている必要はない

- https://www.honeycomb.io/

Rust では `log` クレートを使用すれば、ログレベルを指定してログを収集できるマクロを使用することが可能であり、 `actix-web` を使用している場合には `Logger` ミドルウェアを使用することで HTTP リクエストのログを出力することが可能となる

ただし、 `set_logger` を使用してどこにログ出力を行うのか、という設定をしなければ単にログは破棄されてしまう点に注意が必要となる

以下のようにログレベルを明示的に設定して起動すれば、より詳細なログも出力されるようになっていることがわかる

```bash
RUST_LOG=trace cargo run
```

## ログの一意性

ログを出力するコードを記載したとしても、同時に複数のリクエストを受け取った場合には、処理の順番によっては出力されるログが途中で前後してしまう可能性も存在する

そのため有用なのは以下のようにリクエスト ID を付与した上でログを出力する方法である

```rs
let request_id = Uuid::new_v4();
log::info(
    "request_id {} - Adding '{}' '{}' as a new subscriber",
    request_id,
    form.email,
    form.name
);
```

## ログからトレースへ

ログを考えると、1 つ 1 つの行で定義した段階でそれは独立したログ出力イベントになってしまうが、本来のイベントとしては HTTP リクエストに紐づく形で、リクエストボディのパースや SQL クエリの実行など、階層的に構成される

つまり、1 つ 1 つが独立しているログでは異なる抽象化を行う必要がある

ここで `tracing` クレートの説明を見てみる

> ログメッセージとは異なり、トレースにおけるスパンは開始時刻と終了時刻を有しており、実行の流れに応じて、同じスパンの中でネストされたツリー状にログを収集できる
