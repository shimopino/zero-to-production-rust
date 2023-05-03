use axum::http::StatusCode;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{extract_query_params, setup_app, ConfirmationLinks, TestApp};

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

    let status_code = app.post_newsletters(newsletter_request_body).await;

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
    let status_code = app.post_newsletters(newsletter_request_body).await;

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
        let status_code = app.post_newsletters(invalid_body).await;

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

    let response = app
        .post_newsletters(serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .await;

    assert_eq!(response, StatusCode::UNAUTHORIZED);
    // assert_eq!(r#"Basic realm="publish""#, headers.get("WWW-Authenticate"));
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
