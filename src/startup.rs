use std::net::SocketAddr;

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{
    configuration::{DatabaseSettings, Settings},
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

pub struct Application {
    addr: SocketAddr,
    app: Router,
}

impl Application {
    pub fn build(configuration: Settings) -> Self {
        let connection_pool = get_connection_pool(&configuration.database);

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

        Self {
            app: create_app(app_state),
            addr,
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn app(&self) -> Router {
        self.app.clone()
    }

    pub async fn run_until_stopped(self) -> Result<(), hyper::Error> {
        axum::Server::bind(&self.addr)
            .serve(self.app.into_make_service())
            .await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}
