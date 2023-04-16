use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use sqlx::PgPool;
use tower::ServiceExt;
use zero2prod::configuration::get_configuration;

// #[cfg(feature = "integration_test")]
#[tokio::test]
async fn health_check_works() {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let connection = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let app = zero2prod::startup::create_app(connection);

    let response = app
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
