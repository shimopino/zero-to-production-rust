use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use tower::ServiceExt;

mod common;

use common::setup_app;

// #[cfg(feature = "integration_test")]
#[tokio::test]
async fn health_check_works() {
    let test_app = setup_app().await;

    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .uri("/health_check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"hello world");
}
