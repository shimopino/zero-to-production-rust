use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use crate::{
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
