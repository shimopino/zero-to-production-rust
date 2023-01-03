use std::net::TcpListener;

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
    let server = zero2prod::run(listener).expect("Failed to bind address.");
    // サーバーをバックグラウンドでのタスクとして起動する
    // tokio::spawn はFutureを処理するためのハンドラを返し、テストとして使用する
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
