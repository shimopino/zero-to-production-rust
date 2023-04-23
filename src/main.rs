use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::SocketAddr;
use zero2prod::{
    configuration::get_configuration,
    startup::create_app,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection = PgPool::connect(configuration.database.connection_string().expose_secret())
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
