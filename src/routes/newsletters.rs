use axum::{response::IntoResponse, Json};
use hyper::StatusCode;
use serde::Deserialize;

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

pub async fn publish_subscriber(Json(body): Json<BodyData>) -> impl IntoResponse {
    StatusCode::OK
}
