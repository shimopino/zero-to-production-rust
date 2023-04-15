use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // トップレベルのルーティングを作成する
    let app = Router::new().route("/health_check", get(handler));

    // 実行する
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "hello world"
}
