use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use serde_json::json;
use tower::{Service, ServiceExt};
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{setup_app, TestApp};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let mut app = setup_app().await;
    create_unconfirmed_subscriber(&mut app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act

    let response = app
        .app
        .ready()
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::POST)
                .uri("/newsletters")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "title": "Newsletter title",
                        "content": {
                            "text": "Newsletter body as plain text",
                            "html": "<p>Newsletter body as HTML</p>",
                        }
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
}

async fn create_unconfirmed_subscriber(app: &mut TestApp) {
    let body = "name=shimopino&email=shimopino%40example.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscription(body.into()).await;
}
