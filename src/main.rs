use std::net::SocketAddr;

use env_logger::Env;
use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, startup::create_app};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

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
