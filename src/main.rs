use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_sbscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_sbscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // configurationファイルを読めなければパニックさせる
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let address = TcpListener::bind(address)?;
    run(address, connection_pool)?.await
}
