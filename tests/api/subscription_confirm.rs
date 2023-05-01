use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

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
                .method(http::Method::GET)
                .uri("/subscriptions/confirm")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let mut test_app = setup_app().await;
    let body = "name=shimopino&email=shimopino%40example.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscription(body.into()).await;

    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = test_app.get_confirmation_links(email_request);

    assert_eq!(confirmation_links.html.host_str().unwrap(), "127.0.0.1");

    let query = "subscription_token=mytoken";

    // Act
    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/subscriptions/confirm?{}", query))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let mut test_app = setup_app().await;
    let body = "name=shimopino&email=shimopino%40example.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    // Act
    test_app.post_subscription(body.into()).await;

    // Assert
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = test_app.get_confirmation_links(email_request);

    // 2つのリンクが同じであることを確認する
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}
