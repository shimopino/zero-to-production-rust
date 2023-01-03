#[tokio::test]
async fn health_check_test() {
    // Arrange
    spwan_app().await.expect("Failed to spawn out app.");

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
async fn spwan_app() -> std::io::Result<()> {
    zero2prod::run().await
}
