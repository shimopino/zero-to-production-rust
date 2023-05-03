use std::collections::HashMap;

use axum::{
    body::Body,
    http::{self, HeaderValue, Request},
    Router,
};
use hyper::HeaderMap;
use once_cell::sync::Lazy;
use reqwest::Url;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tower::{Service, ServiceExt};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

/// 確認用のリンクがメールに埋め込まれている
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub app: Router,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscription(&mut self, body: String) -> (axum::http::StatusCode, String) {
        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/subscriptions")
            .header(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .body(Body::from(body))
            .unwrap();

        let response = self
            .app
            .ready()
            .await
            .unwrap()
            .call(request)
            .await
            .expect("Failed to execute request");

        let status = response.status();
        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();

        (status, String::from(body))
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // URLリンクを抽出する
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            // リンクを取得する
            let raw_link = links[0].as_str().to_owned();
            let confirmation_link = reqwest::Url::parse(&raw_link).unwrap();

            // 指定したAPIを実行していることを確認する
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

            confirmation_link
        };

        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }

    pub async fn confirm_link(&mut self, token: String) {
        let request = Request::builder()
            .method(http::Method::GET)
            .uri(format!(
                "/subscriptions/confirm?subscription_token={}",
                token
            ))
            .body(Body::empty())
            .unwrap();

        self.app
            .ready()
            .await
            .unwrap()
            .call(request)
            .await
            .expect("Failed to execute request");
    }

    pub async fn post_newsletters(
        &mut self,
        body: serde_json::Value,
        with_auth_header: bool,
    ) -> (axum::http::StatusCode, HeaderMap) {
        let mut request = Request::builder()
            .method(http::Method::POST)
            .uri("/newsletters")
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        if with_auth_header {
            let auth_value =
                basic_auth_value(Uuid::new_v4().to_string(), Uuid::new_v4().to_string());

            request.headers_mut().insert("Authorization", auth_value);
        }

        let response = self
            .app
            .ready()
            .await
            .unwrap()
            .call(request)
            .await
            .expect("Failed to execute request");

        (response.status(), response.headers().to_owned())
    }
}

pub async fn setup_app() -> TestApp {
    Lazy::force(&TRACING);

    // テスト用のEmailモックサーバー
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone());

    TestApp {
        app: application.app(),
        db_pool: get_connection_pool(&configuration.database),
        email_server,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // データベースを作成する
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // データベースに対してマイグレーションを実行する
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub fn extract_query_params(url: &Url) -> HashMap<String, String> {
    url.query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

pub fn basic_auth_value(username: String, password: String) -> HeaderValue {
    use base64::prelude::BASE64_STANDARD;
    use base64::write::EncoderWriter;
    use std::io::Write;

    let mut buf = b"Basic ".to_vec();
    {
        let mut encoder = EncoderWriter::new(&mut buf, &BASE64_STANDARD);
        let _ = write!(encoder, "{}:", username);
        let _ = write!(encoder, "{}", password);
    }

    let mut header =
        HeaderValue::from_bytes(&buf).expect("Failed to encode base64 authorization header");
    header.set_sensitive(true);
    header
}
