use axum::response::{IntoResponse, Redirect};

pub async fn login() -> impl IntoResponse {
    // https://docs.rs/axum/latest/axum/response/struct.Redirect.html#method.to
    Redirect::to("/")
}
