use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberName};

#[derive(Debug, Deserialize)]
pub struct Subscribe {
    name: String,
    email: String,
}

pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(input): Form<Subscribe>,
) -> impl IntoResponse {
    let name = match SubscriberName::parse(input.name) {
        Ok(name) => name,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let new_subscriber = NewSubscriber {
        email: input.email,
        name,
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            println!("Failed to execute query: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email,
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| e)?;

    Ok(())
}
