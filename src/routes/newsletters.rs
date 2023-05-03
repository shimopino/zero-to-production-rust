use axum::{extract::State, response::IntoResponse, Json};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{error::error_chain_fmt, startup::AppState};

#[derive(Debug, Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        status_code.into_response()
    }
}

pub async fn publish_subscriber(
    State(state): State<AppState>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let _subscribers = get_confirmed_subscribers(&state.db_state.db_pool).await?;

    Ok(StatusCode::OK)
}

struct ConfirmedSubscriber {
    email: String,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    // クエリ内の列の名前が構造体のフィールドと同じであることが期待される
    // 構造体リテラルを使用して行をマッピングする（順序は同じでなくても良い）
    // 列がNULLの可能性がある場合は Option<_> でラップする必要がある
    let rows = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed' "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
