use axum::{
    body::Body,
    extract::State,
    http::{self, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Form,
};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};
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

#[tracing::instrument(
    name = "login",
    skip(form, state),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(State(state): State<AppState>, Form(form): Form<FormData>) -> impl IntoResponse {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &state.db_state.db_pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

            // https://docs.rs/axum/latest/axum/response/struct.Redirect.html#method.to
            Redirect::to("/").into_response()
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            let query_string = format!("error={}", urlencoding::Encoded::new(e.to_string()));

            let hmac_tag = {
                let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
                    state.hmac_secret.0.expose_secret().as_bytes(),
                )
                .unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };

            Response::builder()
                .header(
                    http::header::LOCATION,
                    format!("/login?{query_string}&tag={hmac_tag:x}"),
                )
                .status(StatusCode::SEE_OTHER)
                .body(Body::empty())
                .unwrap()
                .into_response()
        }
    }
}
