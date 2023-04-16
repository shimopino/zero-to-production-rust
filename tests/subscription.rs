use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tower::{Service, ServiceExt};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};

pub struct TestApp {
    pub app: Router,
    pub db_pool: PgPool,
}

pub async fn setup_app() -> TestApp {
    let mut configuration = get_configuration().expect("Failed to read configuration.yml");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let app = zero2prod::startup::create_app(connection_pool.clone());

    TestApp {
        app,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // データベースを作成する
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // データベースに対してマイグレーションを実行する
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_from_data() {
    let test_app = setup_app().await;

    let response = test_app
        .app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/subscriptions")
                .header(
                    http::header::CONTENT_TYPE,
                    mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                )
                .body(Body::from("name=shimopino&email=shimopino%40example.com"))
                .unwrap(),
        )
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::CREATED);

    let saved = sqlx::query!("SELECT * FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "shimopino@example.com");
    assert_eq!(saved.name, "shimopino");
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

    let mut test_app = setup_app().await;

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

        let response = test_app
            .app
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
