use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use tower::ServiceExt;
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
