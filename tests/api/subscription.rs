use crate::helpers::setup_app;
use axum::http::StatusCode;

#[tokio::test]
async fn subscribe_returns_200_for_valid_from_data() {
    let mut test_app = setup_app().await;
    let (status, _) = test_app
        .post_subscription("name=shimopino&email=shimopino%40example.com".to_string())
        .await;

    assert_eq!(status, StatusCode::CREATED);

    let saved = sqlx::query!("SELECT * FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "shimopino@example.com");
    assert_eq!(saved.name, "shimopino");
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
