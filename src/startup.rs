use std::sync::Arc;

use axum::{
    Router,
    http::HeaderName,
    routing::{get, post},
};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};

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
        .layer(
            tower::ServiceBuilder::new()
                // 1️⃣ create request id
                .layer(SetRequestIdLayer::new(
                    HeaderName::from_static("x-request-id"),
                    MakeRequestUuid,
                ))
                // 2️⃣ record request id into tracing spa
                .layer(TraceLayer::new_for_http().make_span_with(
                    |request: &axum::http::Request<_>| {
                        let request_id = request
                            .headers()
                            .get("x-request-id")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("unknown");

                        info_span!(
                            "http_request",
                            method = %request.method(),
                            uri = %request.uri(),
                            request_id = %request_id,
                        )
                    },
                ))
                // 3️⃣ return request id in response
                .layer(PropagateRequestIdLayer::new(HeaderName::from_static(
                    "x-request-id",
                ))),
        );

    let addr = listener.local_addr().expect("Unable to get address");
    info!(%addr, "App running on");

    axum::serve(listener, app).await
}
