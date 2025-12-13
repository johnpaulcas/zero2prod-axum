use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::routes::{health_check, subscribe};

pub struct AppState {
    pub pg_pool: PgPool,
}

pub async fn run(listener: TcpListener, pg_pool: PgPool) -> Result<(), std::io::Error> {
    let shared_state = Arc::new(AppState { pg_pool });

    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(shared_state)
        .layer(TraceLayer::new_for_http());

    let addr = listener.local_addr().expect("Unable to get address");
    info!(%addr, "App running on");

    axum::serve(listener, app).await
}
