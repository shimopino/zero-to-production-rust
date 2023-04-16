use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;

async fn health_check() -> &'static str {
    "hello world"
}

#[derive(Debug, Deserialize)]
struct Subscribe {
    name: String,
    email: String,
}

async fn subscribe(Form(input): Form<Subscribe>) -> impl IntoResponse {
    println!("{}, {}", input.name, input.email);
    StatusCode::CREATED
}

pub fn create_app() -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
}
