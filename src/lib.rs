use actix_web::{web, App, HttpResponse, HttpServer};

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn run() -> std::io::Result<()> {
    // HttpServerはトランスポート層の課題を解決する
    // IPアドレスとポート番号の組み合わせや、最大接続数などを設定できる
    HttpServer::new(|| {
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
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
