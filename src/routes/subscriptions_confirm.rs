use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::error_chain_fmt, startup::AppState};

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmationError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("There is no subscriber associated with the provided token.")]
    UnknownToken,
}

impl std::fmt::Debug for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for ConfirmationError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, error_message) = match self {
            ConfirmationError::UnexpectedError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected Error")
            }
            ConfirmationError::UnknownToken => (StatusCode::UNAUTHORIZED, "Invalid Token"),
        };

        let body = Json(json!({ "error": error_message }));

        (status_code, body).into_response()
    }
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(params, app_state))]
pub async fn confirm(
    State(app_state): State<AppState>,
    Query(params): Query<Parameters>,
) -> Result<impl IntoResponse, ConfirmationError> {
    let subscriber_id =
        get_subscriber_id_from_token(&app_state.db_state.db_pool, &params.subscription_token)
            .await
            .context("Failed to retrieve the subscriber id with token")?
            .ok_or(ConfirmationError::UnknownToken)?;

    confirm_subscriber(&app_state.db_state.db_pool, subscriber_id)
        .await
        .context("Failed to update the subscriber status to 'confirmed'")?;

    Ok(StatusCode::OK)
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
