#[tokio::test]
async fn health_check_test() {
    // Arrange
    spwan_app();

    // HTTPリクエストを送信するためのテストクライアントを初期化
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// アプリケーションコードに依存している箇所は唯一ここだけ
// 内容はほとんど src/main.rs に記載している内容とほとんど同じである
fn spwan_app() {
    // 以下のコードだと、await実行時にサーバーがリッスン状態になってしまいテストが終了しない
    // zero2prod::run().await

    // そこで tokio::spawn を使用してバックグラウンドで起動して
    // Futureを受け取って、その完了を待つ必要なくポーリングできるようにする
    let server = zero2prod::run().expect("Failed to bind address.");
    // サーバーをバックグラウンドでのタスクとして起動する
    // tokio::spawn はFutureを処理するためのハンドラを返し、テストとして使用する
    let _ = tokio::spawn(server);
}
