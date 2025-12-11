use std::{ffi::c_int, fmt::format};

use reqwest::Client;
use sqlx::{Connection, PgConnection};
use zero2prod_axum::{configuration::get_configuration, run};

async fn spawn_app() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind");
    let addr = listener.local_addr().unwrap();

    let server = run(listener);
    tokio::spawn(server);

    format!("http://{}", addr)
}

#[tokio::test]
async fn heath_check() {
    let addr = spawn_app().await;

    let config = get_configuration().expect("Failed to load configuration");
    let url = config.database.connection_string();

    let mut connection = PgConnection::connect(&url)
        .await
        .expect("Failed postgres connection");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health-check", addr))
        .send()
        .await
        .expect("Health Check failed");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn test_valid_form_data() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscription", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn test_400_data_missing() {
    let ip_address = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscription", ip_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
