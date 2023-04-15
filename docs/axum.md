# axum

Web フレームワークとして `axum` を利用する。これは非同期ランタイムである `tokio` を作成しているチームがコミットしているライブラリである。

`tokio` ベースの HTTP サーバーである `tower` などを利用することができる。

## クレートのインストール

今回使用する `axum` と非同期ランタイムである `tokio` をインストールする。

```bash
$ cargo add axum
$ cargo add tokio --features full
```

これで以下のように `Cargo.toml` に指定のクレートが記載されていることがわかる

```toml
[dependencies]
axum = "0.6.15"
tokio = { version = "1.27.0", features = ["full"] }
```

## Hello World

まずは公式のサンプルである [hello-world](https://github.com/tokio-rs/axum/tree/main/examples/hello-world) を参考にヘルスチェックを行うためのエンドポイントを作成する。

```rs
use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // トップレベルのルーティングを作成する
    let app = Router::new().route("/", get(handler));

    // 実行する
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// 単純にテキストを返すだけのエンドポイントを作成する
async fn handler() -> &'static str {
    "hello world"
}
```

これで HTTP リクエストを送信するための拡張機能([http request](../sample/request.http))を利用して、実際に HTTP リクエストを送信すると意図通りのレスポンスが返却されていることがわかる

```bash
### Health Check
GET http://127.0.0.1:8080

### Result
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 11
date: Sat, 15 Apr 2023 09:47:58 GMT

hello world
```

## 細かい実装の中身を見てみる

### Router

[Router](https://docs.rs/axum/latest/axum/routing/struct.Router.html) の役割を確認する

これは HTTP のパスと対応するハンドラーを設定するための構造体であり、ベースとして `hyper` と同じようにハンドラーを設定することができる

```rs
#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));
}

// 単純にテキストを返すだけのエンドポイントを作成する
async fn handler() -> &'static str {
    "hello world"
}
```

このルーティングに関してハンドラー関数は `IntoResponse` を実装した型であれば値を返却することが可能である

今回は `'static str` をレスポンスとして返却しているが、これは公式で以下のようにトレイトを実装しているため、レスポンスに指定することが可能である

```rs
impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Cow::Borrowed(self).into_response()
    }
}


impl IntoResponse for Cow<'static, str> {
    fn into_response(self) -> Response {
        let mut res = Full::from(self).into_response();
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
        );
        res
    }
}
```

- [axum-core/src/response/into_response.rs](https://github.com/tokio-rs/axum/blob/7219fd8df520d295faa42b59f77e25ca2818b6b1/axum-core/src/response/into_response.rs#L232-L236)

ここで使用されている `Cow` は `Borrowed` と `Owned` という 2 つのバリアントを持っている列挙子であり、データへの参照を保持するのか、データの所有権を持つのか選択することが可能である

この実装のおかげで静的な文字列の参照である `'static str` から `Response` に変換することができるようになっており、 `Cow<'static str>` 自体も Response を返却するように実装されていることがわかる

この性質を使えば `IntoResponse` を実装した独自の型を定義することも可能である

### tokio のマクロ

`cargo-expand` パッケージを使用すれば、マクロを展開してどのようなコードが実行されているのかを確認することができる。

```bash
$ cargo install cargo-expand

$ cargo expand
```

10 歳の実行結果を見てみると以下のようになっており、実際にファイル上にコードで記載した部分が `body` 変数に対して代入されていることや、非同期ランタイムの `tokio` のマクロを使用した部分が `Builder` を使用したコードに展開されていることがわかる

```rs
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use axum::{routing::get, Router};
use std::net::SocketAddr;
fn main() {
    let body = async {
        let app = Router::new().route("/", get(handler));
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        {
            ::std::io::_print(format_args!("listening on {0}\n", addr));
        };
        axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
async fn handler() -> &'static str {
    "hello world"
}
```
