use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use axum::Router;
use sqlx::Connection;
use sqlx::Executor;
use sqlx::PgConnection;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use zero2prod::configuration::get_configuration;
use zero2prod::configuration::DatabaseSettings;

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
