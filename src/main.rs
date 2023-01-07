use secrecy::ExposeSecret;
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
    // Poolを初めて使用される時にのみ、接続を確立しようとする
    let connection_pool =
        PgPool::connect_lazy(configuration.database.connection_string().expose_secret())
            .expect("Failed to connect to Postgres");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let address = TcpListener::bind(address)?;
    run(address, connection_pool)?.await
}
