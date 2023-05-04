use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use base64::Engine;
use hyper::{header, HeaderMap, StatusCode};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    domain::SubscriberEmail, error::error_chain_fmt, startup::AppState,
    telemetry::spawn_blocking_with_tracing,
};

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
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
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
        let response = match self {
            PublishError::UnexpectedError(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
            PublishError::AuthError(_) => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header(header::WWW_AUTHENTICATE, r#"Basic realm="publish""#)
                .body(Body::empty())
                .unwrap(),
        };

        response.into_response()
    }
}

#[tracing::instrument(
    name = "Publish a newsletterissue",
    skip(state, headers, body),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_subscriber(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let credentials = basic_authentication(&headers).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &state.db_state.db_pool).await?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    let subscribers = get_confirmed_subscribers(&state.db_state.db_pool).await?;

    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                state
                    .email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    // error chainを構造化ログとして記録する
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                     Their stored contact details are invalid",
                )
            }
        }
    }

    Ok(StatusCode::OK)
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    // query_as!(構造体、クエリ、パラメータ)
    // クエリ内の列の名前が構造体のフィールドと同じであることが期待される
    // 構造体リテラルを使用して行をマッピングする（順序は同じでなくても良い）
    // 列がNULLの可能性がある場合は Option<_> でラップする必要がある

    // ただし、 query! で取得したデータを変更するようにすれば一発で記述可能
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email FROM subscriptions WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();

    Ok(confirmed_subscribers)
}

struct Credentials {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(name = "extract username & password from Authorization")]
fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'")?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-encoded 'Basic' credentials")?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credentials string is not valid UTF8.")?;

    // 仕様に基づいてユーザー名とパスワードを分離
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth"))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth"))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, PublishError> {
    let mut user_id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username, pool)
            .await
            .map_err(PublishError::UnexpectedError)?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task")
    .map_err(PublishError::UnexpectedError)??;

    user_id.ok_or_else(|| PublishError::AuthError(anyhow::anyhow!("Unknown username.")))
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), PublishError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")
        .map_err(PublishError::UnexpectedError)?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(PublishError::AuthError)
}

#[tracing::instrument(name = "Get stored credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE username = $1
        "#,
        username
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to retrieve stored credentials.")?
    .map(|row| (row.user_id, Secret::new(row.password_hash)));

    Ok(row)
}
