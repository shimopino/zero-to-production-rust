use axum::{
    response::{IntoResponse, Redirect},
    Form,
};
use secrecy::Secret;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

pub async fn login(Form(form): Form<FormData>) -> impl IntoResponse {
    // https://docs.rs/axum/latest/axum/response/struct.Redirect.html#method.to
    Redirect::to("/")
}
