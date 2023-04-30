use crate::helpers::setup_app;
use axum::http::StatusCode;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn subscribe_returns_200_for_valid_from_data() {
    let mut test_app = setup_app().await;

    // Emailのモック用サーバーの設定
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let (status, _) = test_app
        .post_subscription("name=shimopino&email=shimopino%40example.com".to_string())
        .await;

    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let mut test_app = setup_app().await;

    // Emailのモック用サーバーの設定
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let (status, _) = test_app
        .post_subscription("name=shimopino&email=shimopino%40example.com".to_string())
        .await;

    assert_eq!(status, StatusCode::CREATED);

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "shimopino@example.com");
    assert_eq!(saved.name, "shimopino");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_400_when_invalid_body() {
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

    let mut test_app = setup_app().await;

    for (invalid_body, error_message) in test_cases {
        let (status, body) = test_app.post_subscription(invalid_body.into()).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, error_message);
    }
}

#[tokio::test]
async fn subscribe_returns_200_when_fields_are_present_but_empty() {
    let mut test_app = setup_app().await;

    let test_cases = vec![
        ("name=&email=shimopino@example.com"),
        ("name=shimopino&email="),
        ("name=shimopino&email=not-an-email"),
    ];

    for empty_body in test_cases {
        let (status, body) = test_app.post_subscription(empty_body.into()).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body, "");
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let mut test_app = setup_app().await;
    let body = "name=shimopino&email=shimopino%40example.com";

    // Email用のモックサーバー
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    test_app.post_subscription(body.into()).await;

    // Assert
    // ドロップ時にMockでの検証も実行される
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let mut test_app = setup_app().await;
    let body = "name=shimopino&email=shimopino%40example.com";

    // Email用のモックサーバー
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // 検証処理は後で実行する
        // .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    test_app.post_subscription(body.into()).await;

    // Assert
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    // URLリンクを抽出する
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(), 1);

        links[0].as_str().to_owned()
    };

    let html_link = get_link(body["HtmlBody"].as_str().unwrap());
    let text_link = get_link(body["TextBody"].as_str().unwrap());

    // 抽出した2つのリンクが同じであること
    assert_eq!(html_link, text_link);
}
