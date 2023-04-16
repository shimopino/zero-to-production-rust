# POST リクエストの作成

- [POST リクエストの作成](#post-リクエストの作成)
  - [考慮すべき内容](#考慮すべき内容)
  - [失敗する結合テスト](#失敗する結合テスト)
  - [必要最小限の実装](#必要最小限の実装)

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
POST http://127.0.0.1:8080/subscription

POST http://127.0.0.1:8080/subscription
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
