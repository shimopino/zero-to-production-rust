use std::net::SocketAddr;

use zero2prod::create_app;

#[tokio::main]
async fn main() {
    let app = create_app();

    // 実行する
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
