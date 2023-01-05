use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // init() を実行することで set_logger が呼び出されて標準出力にログが出力される
    // ログレベルが RUST_LOG 環境変数に設定されていない場合は info レベルを設定する
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // configurationファイルを読めなければパニックさせる
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let address = TcpListener::bind(address)?;
    run(address, connection_pool)?.await
}
