use axum::{routing::get, Router};
use std::net::SocketAddr;

async fn handler() -> &'static str {
    "hello world"
}

pub fn create_app() -> Router {
    let app = Router::new().route("/health_check", get(handler));

    app
}

pub async fn run() {
    let app = create_app();

    // 実行する
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
