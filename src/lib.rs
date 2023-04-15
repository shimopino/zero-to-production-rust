use axum::{routing::get, Router};

async fn handler() -> &'static str {
    "hello world"
}

pub fn create_app() -> Router {
    let app = Router::new().route("/health_check", get(handler));

    app
}
