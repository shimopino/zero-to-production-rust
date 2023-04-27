use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::{create_app, AppState},
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    );

    let app_state = AppState::new(connection, email_client);
    let app = create_app(app_state);

    // 実行する
    let addr = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    )
    .parse()
    .expect("SockerAddr is not valid");

    tracing::info!("{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
