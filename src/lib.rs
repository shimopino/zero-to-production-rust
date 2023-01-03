use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    // HttpServerはトランスポート層の課題を解決する
    // IPアドレスとポート番号の組み合わせや、最大接続数などを設定できる
    let server = HttpServer::new(|| {
        // アプリケーション層の課題を解決する
        // ルーティングやミドルウェア、リクエストハンドラなどを解決する
        // Builder Patternsの実践例であることがわかる
        App::new()
            // ルートパスとリクエストハンドラのペアを登録する
            // web::get() == Route::new().guard(guard::Get())
            // .route("/", web::get().to(greet))
            // .route("/{name}", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    // .bind(address)?
    .run();
    // awaitでリッスン状態にするのではなく、初期化した状態のサーバーを返却するように変更する
    // .await

    Ok(server)
}
