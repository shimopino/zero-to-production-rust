use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Subscribe {
    name: String,
    email: String,
}

pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(input): Form<Subscribe>,
) -> impl IntoResponse {
    if !is_valid_name(&input.name) {
        return StatusCode::BAD_REQUEST;
    }

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        input.email,
        input.name,
        Utc::now()
    )
    .execute(&pool)
    .await
    {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            println!("Failed to execute query: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub fn is_valid_name(s: &str) -> bool {
    let is_empty_or_whitespace = s.trim().is_empty();

    let is_too_long = s.graphemes(true).count() > 256;

    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}
