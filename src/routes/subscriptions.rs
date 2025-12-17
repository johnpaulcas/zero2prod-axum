use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::domain::NewSubscriber;
use crate::domain::SubsciberName;
use crate::domain::SubscriberEmail;
use crate::startup::AppState;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::parse(value.email)?;
        let name = SubsciberName::parse(value.name)?;

        Ok(Self { email, name })
    }
}

#[tracing::instrument(
    name = "Adding new subscriber",
    skip_all,
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    let new_subscriber = match form.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match insert_subscriber(&state.pg_pool, &new_subscriber).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[tracing::instrument(name = "Saving new subscriber detalis to the database", skip_all)]
pub async fn insert_subscriber(pool: &PgPool, form: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, name, email, subscribed_at)
        VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.name.as_ref(),
        form.email.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
