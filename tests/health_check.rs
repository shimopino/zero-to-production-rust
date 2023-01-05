use std::net::TcpListener;

use sqlx::{Connection, PgConnection};
use zero2prod::configuration::get_configuration;

#[tokio::test]
async fn health_check_test() {
    // Arrange
    let address = spwan_app();

    // HTTPリクエストを送信するためのテストクライアントを初期化
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let address = spwan_app();
    // DBに接続する
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");
    // Httpリクエストを送信するためのクライアントを作成
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

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let address = spwan_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencode")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Arrange
        assert_eq!(
            400,
            response.status().as_u16(),
            // テスト失敗時に表示するメッセージをカスタマイズする
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        )
    }
}

// アプリケーションコードに依存している箇所は唯一ここだけ
// 内容はほとんど src/main.rs に記載している内容とほとんど同じである
fn spwan_app() -> String {
    // 以下のコードだと、await実行時にサーバーがリッスン状態になってしまいテストが終了しない
    // zero2prod::run().await

    // 接続用のリスナーを用意
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // OSから割り当てられているポート番号を取得する
    let port = listener.local_addr().unwrap().port();

    // そこで tokio::spawn を使用してバックグラウンドで起動して
    // Futureを受け取って、その完了を待つ必要なくポーリングできるようにする
    // ここではポート番号をライブラリなどから取得する必要がある
    let server = zero2prod::startup::run(listener).expect("Failed to bind address.");
    // サーバーをバックグラウンドでのタスクとして起動する
    // tokio::spawn はFutureを処理するためのハンドラを返し、テストとして使用する
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
