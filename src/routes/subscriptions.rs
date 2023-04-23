use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(Debug, Deserialize)]
pub struct Subscribe {
    name: String,
    email: String,
}

impl TryFrom<Subscribe> for NewSubscriber {
    type Error = String;

    fn try_from(value: Subscribe) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(form): Form<Subscribe>,
) -> impl IntoResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    );
    let _request_span_guard = request_span.enter();

    let new_subscriber = match form.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            StatusCode::CREATED
        }
        Err(e) => {
            tracing::error!("request_id {} - Failed to execute query: {}", request_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    let query_span = tracing::info_span!("Saving new subscriber details in the database");

    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .instrument(query_span)
    .await?;

    Ok(())
}
