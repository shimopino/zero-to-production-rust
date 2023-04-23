use sqlx::PgPool;
use std::net::SocketAddr;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zero2prod::{configuration::get_configuration, startup::create_app};

#[tokio::main]
async fn main() {
    // Logで出力されている内容をSubscriberに転送する
    LogTracer::init().expect("Failed to set logger");

    // デフォルトでは INFO レベル以上のログを出力する
    // 環境変数から設定できるように EnvFilter を利用する
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // フォーマットされた span を標準出力に出す
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);

    // with メソッドを使用して機能を拡張する
    // Registryのおかげで複数の機能を簡単に組み合わせることが可能
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // 実行する
    let addr = SocketAddr::from(([127, 0, 0, 1], configuration.application_port));

    let app = create_app(connection);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
