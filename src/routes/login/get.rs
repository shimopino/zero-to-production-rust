use axum::{
    extract::{Query, State},
    response::Html,
};
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use serde::Deserialize;

use crate::startup::{AppState, HmacSecret};

#[derive(Deserialize)]
pub struct QueryParams {
    error: Option<String>,
    tag: Option<String>,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = match self.tag {
            None => return Ok("".into()),
            Some(tag) => hex::decode(tag),
        }?;

        let (query_string, error) = match self.error {
            None => return Ok("".into()),
            Some(error) => (
                format!("error={}", urlencoding::Encoded::new(&error)),
                error,
            ),
        };

        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(error)
    }
}

pub async fn login_form(
    State(state): State<AppState>,
    Query(query): Query<QueryParams>,
) -> Html<String> {
    let error_html = match query.verify(&state.hmac_secret) {
        Ok(error) => {
            format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error))
        }
        Err(e) => {
            tracing::warn!(
            error.message = %e,
            error.cause_chain = ?e,
            "Failed to verify query parameters using the HMAC tag"
            );
            "".into()
        }
    };

    Html(format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Login</title>
        </head>
        <body>
            {error_html}
            <form action="/login" method="post">
                <label>Username
                    <input
        type="text"
        placeholder="Enter Username"
                name="username"
            >
        </label>
        <label>Password
            <input
                type="password"
                placeholder="Enter Password"
                name="password"
            >
        </label>
                <button type="submit">Login</button>
            </form>
        </body> </html>"#
    ))
}
