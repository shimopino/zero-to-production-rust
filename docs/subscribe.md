# POST リクエストの作成

- [POST リクエストの作成](#post-リクエストの作成)
  - [考慮すべき内容](#考慮すべき内容)
  - [失敗する結合テスト](#失敗する結合テスト)
  - [必要最小限の実装](#必要最小限の実装)
  - [リクエストボディに対する検証を行うテストを追加](#リクエストボディに対する検証を行うテストを追加)
  - [リクエストボディを検証する処理を追加](#リクエストボディを検証する処理を追加)
    - [リクエストボディの取り扱い](#リクエストボディの取り扱い)
    - [注意点](#注意点)
    - [axum 側の実装](#axum-側の実装)
  - [データベースへの接続](#データベースへの接続)
    - [sqlx を利用したアクセス](#sqlx-を利用したアクセス)
    - [sqlx を利用して DB 接続する](#sqlx-を利用して-db-接続する)
    - [テストでのデータベース確認](#テストでのデータベース確認)
    - [PgConnection と PgPool](#pgconnection-と-pgpool)
    - [テストとデータベース](#テストとデータベース)
  - [Github Actions でテストを実現する](#github-actions-でテストを実現する)

## 考慮すべき内容

ニュースを購読するためのエンドポイント( `POST /subscription` )を作成するにあたって、以下の点を考慮する必要がある

- HTML フォームのデータをどのように受け取るのか
- PostgreSQL インどのように接続するのか
- マイグレーションの設定
- DB 接続の管理方法
- テストの副作用の取り扱い
- テストとデータベース間の影響を避ける

以下の期待値を達成できる機能を作成する

```txt
### Emailを登録してニュールを購読する
POST http://127.0.0.1:8080/subscriptions
Content-Type: application/x-www-form-urlencoded

name=shimopino
&email=shimopino@example.com
```

今回は `application/json` 形式ではなく、HTML で構築された `form` をそのまま利用した時の送信形式である `application/x-www-form-urlencoded` を利用する

## 失敗する結合テスト

`axum` の公式 Github に挙げられているサンプルコードには POST リクエストを行うコードも配置されているため、このコードを参考に失敗するテストケースを作成する

- [testing](https://github.com/tokio-rs/axum/blob/main/examples/testing/src/main.rs)

上記のコードでは `mime` クレートを利用しているため、テスト用にインストールしておく

```bash
$ cargo add --dev mime
```

ここでは以下のようなコードとなる

```rs
let app = create_app();

let response = app
    .oneshot(
        Request::builder()
            .method(http::Method::POST)
            .uri("/subscriptions")
            .header(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .body(
                Body::from(
                    "name=shimopino&email=shimopino%40example.com",
                )
            )
            .unwrap(),
    )
    .await
    .expect("Failed to execute request");

assert_eq!(response.status(), StatusCode::CREATED);
```

公式のコード補完では以下のように HTTP ヘッダーをテキストベースで指定することも可能だが、今回は公式サイトのサンプル通りにそれぞれのクレートが提供している定数を利用する

```rs
let req = Request::builder()
    .header("Accept", "text/html")
    .header("X-Custom-Foo", "bar")
    .body(())
    .unwrap();
```

これでテストを実行すれば以下のようにエンドポイントを作成していないため、NOT FOUND のステータスコードが返却されていることがわかる

```bash
---- subscribe_returns_200_for_valid_from_data stdout ----
thread 'subscribe_returns_200_for_valid_from_data' panicked at 'assertion failed: `(left == right)`
  left: `404`,
 right: `201`', tests/subscription.rs:29:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## 必要最小限の実装

テストを合格させる必要最小限の実装を行うために、新しく該当のエンドポイントを作成し、合致するステータスコードを返却するように修正する

```rs
async fn subscribe() -> impl IntoResponse {
    StatusCode::CREATED
}

pub fn create_app() -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
}
```

これでテストを実行すれば PASS することがわかる

## リクエストボディに対する検証を行うテストを追加

今回はリクエストボディに対してニュースを購読するユーザーの名前とメールアドレスを指定する必要があるが、これらを必須の入力項目として取り扱う

そのためそれぞれの入力項目が指定されていない場合に BAD REQUEST を返すことを前提とした失敗するテストを追加する

```rs
#[tokio::test]
async fn subscribe_returns_400_when_invalid_body() {
    // 複数のテストケースを用意する
    let test_cases = vec![
        ("name=shimopino", "email is missing"),
        ("email=shimopino%40example.com", "name is missing"),
        ("", "name and email are missing"),
    ];

    // 複数回りクエストを送信できるように可変状態で作成する
    let mut app = create_app();

    for (invalid_body, error_message) in test_cases {
        let request = Request::builder()
            .method(http::Method::POST)
            .header(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .uri("/subscriptions")
            .body(Body::from(invalid_body))
            .unwrap();

        let response = app
            .ready()
            .await
            .unwrap()
            .call(request)
            .await
            .expect("Failed to execute request");

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "payload was {}",
            error_message
        );
    }
}
```

配列を使用することで複数のテストを実行するためのデータを用意している

`ready` と `call` のメソッドを利用することでループの中で `clone` を利用する必要がない状態にしておき、テスト実行時のパフォーマンスを向上させている

## リクエストボディを検証する処理を追加

`application/x-www-url-encoded` 形式の HTTP リクエストを取り扱う上で、 `axum::Form` を利用することが可能である

- [axum::Form](https://docs.rs/axum/latest/axum/struct.Form.html)

この構造体を利用することで `application/x-www-url-encoded` 形式でエンコーディングされているリクエストボディをデシリアライズすることが可能であり、 `serde::Deserialize` を実装している全ての型をサポートしている

こうしたリクエスト情報からデータを抽出する機能は `Extractor` と呼ばれており、各ハンドラーの引数に指定することでデータを抽出する

ただし、Extractor は関数の引数に対して左から右へ順番に適用され、リクエストボディは非同期ストリームであり一度しか消費することができない

そのためリクエストボディを消費する Extractor は関数の最後の引数に指定する必要がある

- [Extractor の順番](https://docs.rs/axum/latest/axum/extract/index.html#the-order-of-extractors)

まずはデシリアライズのできるように `serde` をインストールする

```bash
$ cargo add serde --features derive
```

### リクエストボディの取り扱い

公式サイトに従って以下のようにリクエストボディとハンドラーを定義する

```rs
#[derive(Debug, Deserialize)]
struct Subscribe {
    name: String,
    email: String,
}

async fn subscribe(Form(input): Form<Subscribe>) -> impl IntoResponse {
    println!("{}, {}", input.name, input.email);
    StatusCode::CREATED
}
```

これでリクエストを送信すれば以下のようにログが出力されていることがわかる

```bash
listening on 127.0.0.1:8080
shimopino, shimopino@example.com
```

### 注意点

ただしテストを実行すると以下のように意図したステータスコードではないことがわかる

```bash
---- subscribe_returns_400_when_invalid_body stdout ----
thread 'subscribe_returns_400_when_invalid_body' panicked at 'assertion failed: `(left == right)`
  left: `422`,
 right: `404`: payload was email is missing', tests/subscription.rs:61:9
```

これはより詳細にエラーメッセージを見ると以下のようになっている

```bash
HTTP/1.1 422 Unprocessable Entity
content-type: text/plain; charset=utf-8
content-length: 54
date: Sun, 16 Apr 2023 07:20:41 GMT

Failed to deserialize form body: missing field `email`
```

確かにテストケースで想定していた通りのエラーが出力されていることがわかるが、ライブラリが出したエラーをそのまま出力していることがわかる

### axum 側の実装

今回使用した `Form` 構造体は公式では下記ファイルで定義されている

- [Form](https://github.com/tokio-rs/axum/blob/main/axum/src/form.rs)

ルーティング設定を行なったときに定義したハンドラー関数は `FromRequest` トレイトを実装したものを指定する必要があり、このトレイトは以下のように定義されている

```rs
// axum-core/src/extract/mod.rs
pub trait FromRequest<S, B, M = private::ViaRequest>: Sized {
    /// リクエストの検証が Extractor で失敗して場合には Rejection を使ってレスポンスを返す
    type Rejection: IntoResponse;

    /// Perform the extraction.
    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection>;
}
```

そして Form は以下のようにリクエストボディを処理している

```rs
#[async_trait]
impl<T, S, B> FromRequest<S, B> for Form<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = FormRejection;

    async fn from_request(req: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        // x-www-url-encodedなので受け付けることの可能なHTTPメソッドが限定されている
        let is_get_or_head =
            req.method() == http::Method::GET || req.method() == http::Method::HEAD;

        match req.extract().await {
            // 非同期ストレージとして Bytes 型としてリクエストを受け取り
            Ok(RawForm(bytes)) => {
                // serde_urlencoded で Bytes からデシリアライズを行なっている
                let value =
                    serde_urlencoded::from_bytes(&bytes).map_err(|err| -> FormRejection {
                        if is_get_or_head {
                            FailedToDeserializeForm::from_err(err).into()
                        } else {
                            FailedToDeserializeFormBody::from_err(err).into()
                        }
                    })?;
                Ok(Form(value))
            }
            // もしもエラーが発生した場合にはRejectionを返す
            Err(RawFormRejection::BytesRejection(r)) => Err(FormRejection::BytesRejection(r)),
            Err(RawFormRejection::InvalidFormContentType(r)) => {
                Err(FormRejection::InvalidFormContentType(r))
            }
        }
    }
}
```

今回はエラーメッセージのカスタマイズを後回しにして、テストケース側を修正することで対応する

```rs
let test_cases = vec![
    (
        "name=shimopino",
        "Failed to deserialize form body: missing field `email`",
    ),
    (
        "email=shimopino%40example.com",
        "Failed to deserialize form body: missing field `name`",
    ),
    ("", "Failed to deserialize form body: missing field `name`"),
];

// ...

assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
// Bytes型に対して Deref トレイトの実装を呼び出すことで変換している
// Deref<Target = [u8]> が実装されているため自動的に &[u8] に変換される
let body = std::str::from_utf8(bytes.as_ref());

assert_eq!(body, Ok(error_message));
```

## データベースへの接続

Rust にてデータベースを使用するライブラリを選択する際には、以下の観点が重要となる

- コンパイル時の型安全性
- クエリを利用する時のインターフェース
- 非同期サポート

本書では `sqlx` を採用しており、 `axum` にもサンプルはあるためこのライブラリを使用する

- [sqlx](https://github.com/launchbadge/sqlx)

結合テストでデータベースを使用する際には、検証のために ① 別のエンドポイントを使用して状態を確認するか、② データベースに直接アクセスして確認する方法が存在している

将来のリファクタリングのためには実装の詳細を把握する必要がないように ① を選択する方が望ましいが、別の機能を作成することになるため一旦 ② の実装で進めていく

今回はローカルで開発する際には Docker を利用して Postgres サーバーを使用するため、 `scripts/init_db.sh` を実行してコンテナを起動する

```bash
$ chmod +x scripts/init_db.sh

# PostgreSQLサーバーを起動する
$ ./scripts/init_db.sh

# 実際にコンテナにアクセスしてデータベースを確認する
$ docker container exec -it postgres bash
> psql -h localhost -U $POSTGRES_USER -d $POSTGRES_DB -W
> \dt -- テーブルの一覧を表示する
```

### sqlx を利用したアクセス

まずは `sqlx` をインストールする

```bash
# sqlxをインストールする
$ cargo install sqlx-cli --no-default-features --features rustls,postgres

# バージョン情報を確認する
$ sqlx --version
sqlx-cli 0.6.3
```

sqlx では以下のようにデータベース作成のために接続 URL を指定する

```bash
sqlx database create --help
sqlx-database-create
Creates the database specified in your DATABASE_URL

USAGE:
    sqlx database create [OPTIONS] --database-url <DATABASE_URL>

OPTIONS:
        --connect-timeout <CONNECT_TIMEOUT>
            The maximum time, in seconds, to try connecting to the database server before returning
            an error [default: 10]

    -D, --database-url <DATABASE_URL>
            Location of the DB, by default will be read from the DATABASE_URL env var [env:
            DATABASE_URL=]
```

あとは `sqlx` を使用してニュースの購読に必要なテーブルのマイグレーションファイルを作成するために下記コマンドを実行すれば `migrations` ディレクトリに空の SQL ファイルが作成されていることが確認できる

```bash
$ DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/newsletter sqlx migrate add create_subscriptions_table

Creating migrations/20230416091820_create_subscriptions_table.sql

Congratulations on creating your first migration!
...
```

以下の通りメールアドレスには一意性制約をもうけたテーブルを作成する

```sql
CREATE TABLE subscriptions(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL
)
```

最後にデータベース初期化用のスクリプトに対してマイグレーションを実行する下記コマンドを追加する

```bash
sqlx database create
sqlx migrate run # このコマンドを追加
```

これでデータベースにアクセスすれば以下のようにテーブルが作成されていることが確認できる

```bash
# 実際にコンテナにアクセスしてデータベースを確認する
$ docker container exec -it postgres bash
> psql -h localhost -U $POSTGRES_USER -d $POSTGRES_DB -W
newsletter=# \dt -- テーブルの一覧を表示する
              List of relations
 Schema |       Name       | Type  |  Owner
--------+------------------+-------+----------
 public | _sqlx_migrations | table | postgres
 public | subscriptions    | table | postgres
(2 rows)

newsletter=# \d subscriptions
                       Table "public.subscriptions"
    Column     |           Type           | Collation | Nullable |
 Default
---------------+--------------------------+-----------+----------+
---------
 id            | uuid                     |           | not null |

 email         | text                     |           | not null |

 name          | text                     |           | not null |

 subscribed_at | timestamp with time zone |           | not null |

Indexes:
    "subscriptions_pkey" PRIMARY KEY, btree (id)
    "subscriptions_email_key" UNIQUE CONSTRAINT, btree (email)

newsletter=# \d _sqlx_migrations
                      Table "public._sqlx_migrations"
     Column     |           Type           | Collation | Nullable | Default
----------------+--------------------------+-----------+----------+---------
 version        | bigint                   |           | not null |
 description    | text                     |           | not null |
 installed_on   | timestamp with time zone |           | not null | now()
 success        | boolean                  |           | not null |
 checksum       | bytea                    |           | not null |
 execution_time | bigint                   |           | not null |
Indexes:
    "_sqlx_migrations_pkey" PRIMARY KEY, btree (version)
```

### sqlx を利用して DB 接続する

sqlx をインストールするが、指定する Feature Flag が多く複雑になるため、以下のように別途依存関係の設定を定義することができる

```toml
[dependencies.sqlx]
version = "^0.6"
default-features = false
features = [
    # tokioランタイムとTLSバックエンドを利用する
    "runtime-tokio-rustls",
    # sqlx::query!などのマクロを利用する
    "macros",
    # Postgres特有の機能を利用する
    "postgres",
    # SQLのUUIDマッピング機能を利用する
    "uuid",
    # SQL timestamptzをDateTime<T>にマッピングする機能を利用する
    "chrono",
    # sqlx-cliのマイグレーションなどの機能を利用する
    "migrate"
]
```

データベースに接続するには接続文字列を利用する必要があるが、現状のディレクトリ構成のままでは複雑になってしまうため、以下のようなディレクトリ構造を採用して設定ファイルを別個定義する

```bash
src/
├── configuration.rs        # 各種アプリケーション設定を行う
├── lib.rs                  # ライブラリをpublishさせるのみ
├── main.rs
├── routes
│   ├── health_check.rs
│   ├── mod.rs
│   └── subscriptions.rs
└── startup.rs              # サーバー起動に必要な関数をまとめる
```

Rust では `yml` ファイルなどで設定されたアプリケーション設定の値を読み込むための [config-rs](https://github.com/mehcode/config-rs) クレートを使用することができ、ファイルで設定している値と同じ構造の型を用意すればいい

例えば `configuration.yml` で以下のように値を設定した場合

```yml
application_port: 8080
database:
  host: "127.0.0.1"
  port: 5432
  username: "postgres"
  password: "password"
  database_name: "newsletter"
```

Rust では以下のように型を定義する

```rs
#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}
```

そしてクレートを使用してパースすればファイルに設定された項目を読み込むことが可能となる

```rs
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}
```

- [config-rs example](https://github.com/mehcode/config-rs/blob/master/examples/simple/main.rs)

こうしておけば DB への接続文字列を生成するための関数を作成することも可能である

```rs
impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}
```

### テストでのデータベース確認

結合テストで DB に対して直接クエリを実行することができるが、公式ページに記載されている通り環境変数などを設定する必要がある

- [sqlx::query!](https://docs.rs/sqlx/latest/sqlx/macro.query.html#requirements)

以下の点を気をつける必要がある

- ビルド時に `DATABASE_URL` という環境変数が設定されている必要があり、クエリ文字列が指定されたデータベースサーバーに対して検証を行うために設定する
  - `.env` ファイルに環境変数を設定する
  - オフラインモードで利用可能な `sql-data.json` をワークスペースルートに設定する

今回は `.env` に `DATABASE_URL` のみを配置し、環境変数ファイルは直接コミットせずに CI 環境では `.env.example` からコピーしてコンパイルエラーが発生しないようにする

sqlx を直接利用して、以下のようにデータを取得してその検証処理を入れている

```rs
let saved = sqlx::query!("SELECT * FROM subscriptions",)
    .fetch_one(&test_app.db_pool)
    .await
    .expect("Failed to fetch saved subscription.");

assert_eq!(saved.email, "shimopino@example.com");
assert_eq!(saved.name, "shimopino");
```

### PgConnection と PgPool

sqlx では `Pool` という構造体が定義されており、 `Send + Sync + Clone` が実装されているためプロセスの寿命を通じてさまざまなタスクで共有することが可能である

`Pool` は内部のプールへの参照カウントされた値であるため clone が安価に実現でき、プールへの残りのハンドルがドロップされると、所有されている接続もドロップにより切断される仕組みとなっている

- [PgConnection](https://docs.rs/sqlx/latest/sqlx/struct.PgConnection.html)
- [Pool](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html)

コネクションではなくコネクションプールを利用することで、コネクションの再利用を実現し、以下の追加コストを支払う必要がなくなる

- コネクション接続の開始にかかるオーバーヘッド
  - SQLite であればファイルシステムへの多数のリクエストとメモリ割り当てのコストが発生し、サーバーベースの DB であれば DNS 解決の実行や TCP 接続の開始、バッファ割り当てのコストが発生する
  - 認証や接続パラメータに関するネゴシエーションなどの追加のハンドシェイクが発生する
  - コネクションプールであれば、事前に指定した数の接続を作成することでこうしたオーバーヘッドを支払う必要がない
- 接続制限
  - サーバーベースの DB ではリソースの過剰割り当てを防ぐために、任意の時間に開始できる接続数に制限が設けられている
  - Postgres の場合にはデフォルトの上限は 100 が設定されており、スーパーユーザー用に予約された数の 3 を抜いた 97 だけ接続できる
  - コネクションプールを利用することで、サーバーへの接続制限を超えないようにしながら、高負荷時に応答時間を Gracefully に低下させる
- リソースの再利用
  - データベースに対して初めてクエリを実行するとき、SQL クエリの解析や検証、分析、クエリプランを実行するコードの生成などを行なっている
  - DB サーバーはこのクエリプランをキャッシュに置き、後で実行するときに参照することで上記のオーバーヘッドを減らすようにしている
  - データベース接続は通常 DB サーバー内で分離され、ステートメントを共有しないため、上記のメリットを受けるにはコネクションプールを利用してリソースを可能な限り再利用する

### テストとデータベース

## Github Actions でテストを実現する
