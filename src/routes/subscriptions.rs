use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::startup::AppState;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
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
    match insert_subscriber(&state.pg_pool, &form).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[tracing::instrument(name = "Saving new subscriber detalis to the database", skip_all)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, name, email, subscribed_at)
        VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.name,
        form.email,
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
