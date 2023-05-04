use axum::{
    body::Body,
    extract::State,
    http::{self, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Form,
};
use secrecy::Secret;
use serde::Deserialize;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    error::error_chain_fmt,
    startup::AppState,
};

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        let encoded_error = urlencoding::Encoded::new(self.to_string());

        Response::builder()
            .header(
                http::header::LOCATION,
                format!("/login?error={}", encoded_error),
            )
            .status(StatusCode::SEE_OTHER)
            .body(Body::empty())
            .unwrap()
            .into_response()
    }
}

#[tracing::instrument(
    name = "login",
    skip(form, state),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(state): State<AppState>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &state.db_state.db_pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
        })?;

    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    // https://docs.rs/axum/latest/axum/response/struct.Redirect.html#method.to
    Ok(Redirect::to("/"))
}
