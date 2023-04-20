use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use tower::{Service, ServiceExt};

mod common;

use common::setup_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_from_data() {
    let test_app = setup_app().await;

    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/subscriptions")
                .header(
                    http::header::CONTENT_TYPE,
                    mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                )
                .body(Body::from("name=shimopino&email=shimopino%40example.com"))
                .unwrap(),
        )
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::CREATED);

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
        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/subscriptions")
            .header(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .body(Body::from(invalid_body))
            .unwrap();

        let response = test_app
            .app
            .ready()
            .await
            .unwrap()
            .call(request)
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = std::str::from_utf8(bytes.as_ref());

        assert_eq!(body, Ok(error_message));
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

    for (body) in test_cases {
        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/subscriptions")
            .header(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .body(Body::from(body))
            .unwrap();

        let response = test_app
            .app
            .ready()
            .await
            .unwrap()
            .call(request)
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
