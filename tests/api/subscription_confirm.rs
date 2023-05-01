use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use reqwest::Url;
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

    let raw_confirmation_link = &get_link(body["HtmlBody"].as_str().unwrap());
    let confirmation_link = Url::parse(raw_confirmation_link).unwrap();
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

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
    assert_eq!(response.status(), StatusCode::OK);
}
