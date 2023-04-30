use std::net::SocketAddr;

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{
    configuration::Settings,
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

#[derive(Clone)]
pub struct AppState {
    db_state: DbState,
    email_client: EmailClient,
}

#[derive(Clone)]
pub struct DbState {
    pub db_pool: PgPool,
}

impl AppState {
    pub fn new(db_pool: PgPool, email_client: EmailClient) -> Self {
        Self {
            db_state: DbState { db_pool },
            email_client,
        }
    }
}

impl FromRef<AppState> for DbState {
    fn from_ref(app_state: &AppState) -> DbState {
        app_state.db_state.clone()
    }
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(state)
}

pub fn build(configuration: Settings) -> (Router, SocketAddr) {
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let app_state = AppState::new(connection_pool, email_client);

    // 実行する
    let addr = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    )
    .parse()
    .expect("SockerAddr is not valid");

    (create_app(app_state), addr)
}
