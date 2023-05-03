use axum::response::IntoResponse;
use hyper::StatusCode;

pub async fn publish_subscriber() -> impl IntoResponse {
    StatusCode::OK
}
