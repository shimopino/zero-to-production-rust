use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use tower::{Service, ServiceExt};
use zero2prod::create_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_from_data() {
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
                .body(Body::from(
                    "name=le%20guin&email=ursula_le_guin%40gmail.com",
                ))
                .unwrap(),
        )
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::CREATED);
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

    let mut app = create_app();

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

        let response = app
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
