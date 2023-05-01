use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::startup::AppState;

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(params, app_state))]
pub async fn confirm(
    State(app_state): State<AppState>,
    Query(params): Query<Parameters>,
) -> impl IntoResponse {
    let id =
        match get_subscriber_id_from_token(&app_state.db_state.db_pool, &params.subscription_token)
            .await
        {
            Ok(id) => id,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        };

    match id {
        None => StatusCode::UNAUTHORIZED,
        Some(subscriber_id) => {
            if confirm_subscriber(&app_state.db_state.db_pool, subscriber_id)
                .await
                .is_err()
            {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(pool, subscriber_id))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
        "#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;

    Ok(())
}

pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id FROM subscription_tokens
        WHERE subscription_token = $1
        "#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}
