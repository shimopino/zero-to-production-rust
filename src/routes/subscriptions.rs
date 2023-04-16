use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
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
    println!("{}, {}", input.name, input.email);

    sqlx::query!(
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
    .await;

    StatusCode::CREATED
}
