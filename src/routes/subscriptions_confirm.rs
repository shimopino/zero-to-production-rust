use axum::{extract::Query, response::IntoResponse};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(params))]
#[allow(unused_variables)]
pub async fn confirm(Query(params): Query<Parameters>) -> impl IntoResponse {
    axum::http::StatusCode::OK
}
