use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use tower::ServiceExt;

use crate::helpers::setup_app;

#[tokio::test]
async fn confrmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let test_app = setup_app().await;

    // Act
    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/subscriptions/confirm")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
