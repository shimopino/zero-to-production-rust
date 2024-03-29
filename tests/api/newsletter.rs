use axum::{
    body::Body,
    http::{self, StatusCode},
};
use hyper::Request;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{
    basic_auth_value, extract_query_params, setup_app, ConfirmationLinks, TestApp,
};

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
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });

    let (status_code, _) = app.post_newsletters(newsletter_request_body, true).await;

    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let mut app = setup_app().await;
    create_confirmed_subscriber(&mut app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    let (status_code, _) = app.post_newsletters(newsletter_request_body, true).await;

    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let mut app = setup_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>"
                }
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let (status_code, _) = app.post_newsletters(invalid_body, true).await;

        // Assert
        assert_eq!(
            status_code,
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let mut app = setup_app().await;

    // Act
    let (status_code, headers) = app
        .post_newsletters(
            serde_json::json!({
                "title": "Newsletter title",
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>"
                }
            }),
            // ここだけメソッドを使用せずにそのままリクエストしてしまっても良さそう
            false,
        )
        .await;

    assert_eq!(status_code, StatusCode::UNAUTHORIZED);
    assert_eq!(
        headers.get("WWW-Authenticate").unwrap(),
        &r#"Basic realm="publish""#
    );
}

#[tokio::test]
async fn non_existing_user_is_rejected() {
    // Arrange
    let app = setup_app().await;

    // Random Credentials
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();
    let basic_auth = basic_auth_value(&username, &password);

    // Act
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/newsletters")
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .header(http::header::AUTHORIZATION, basic_auth)
        .body(Body::from(
            serde_json::to_vec(&json!({
                "title": "Newsletter title",
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>"
                }
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app
        .app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get("WWW-Authenticate").unwrap(),
        &r#"Basic realm="publish""#
    )
}

#[tokio::test]
async fn invalid_password_is_rejected() {
    // Arrange
    let app = setup_app().await;
    let username = &app.test_user.username;
    // Random Password
    let password = Uuid::new_v4().to_string();
    assert_ne!(app.test_user.password, password);

    let basic_auth = basic_auth_value(username, &password);

    // Act
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/newsletters")
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .header(http::header::AUTHORIZATION, basic_auth)
        .body(Body::from(
            serde_json::to_vec(&json!({
                "title": "Newsletter title",
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>"
                }
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app
        .app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get("WWW-Authenticate").unwrap(),
        &r#"Basic realm="publish""#
    )
}

async fn create_unconfirmed_subscriber(app: &mut TestApp) -> ConfirmationLinks {
    let body = "name=shimopino&email=shimopino%40example.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscription(body.into()).await;

    // Emailサーバーに送信されたメールから本文を抽出する
    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    // メール本文からURLリンクを抽出する
    app.get_confirmation_links(email_request)
}

async fn create_confirmed_subscriber(app: &mut TestApp) {
    // リンクからトークンを抽出して送信することで、確認済みデータを作成する
    let confirmation_links = create_unconfirmed_subscriber(app).await;

    let query_params = extract_query_params(&confirmation_links.html);
    let token = query_params.get("subscription_token").unwrap().to_owned();

    // Act
    app.confirm_link(token).await;
}
