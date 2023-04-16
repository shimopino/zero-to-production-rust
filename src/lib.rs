use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

async fn health_check() -> &'static str {
    "hello world"
}

async fn subscribe() -> impl IntoResponse {
    StatusCode::CREATED
}

pub fn create_app() -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
}
