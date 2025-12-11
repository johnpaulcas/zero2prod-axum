pub mod configuration;
pub mod routes;
pub mod startup;

use axum::{
    Form, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use tokio::net::TcpListener;

#[derive(serde::Deserialize, Debug)]
struct FormData {
    email: String,
    name: String,
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn subscribe(Form(_form): Form<FormData>) -> impl IntoResponse {
    println!("{:?}", _form);
    StatusCode::OK
}

pub fn app() -> Router {
    Router::new().route("/health-check", get(health_check))
}

pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/subscription", post(subscribe));

    axum::serve(listener, app).await
}
