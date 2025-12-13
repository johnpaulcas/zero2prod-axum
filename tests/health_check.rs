use std::{env, sync::LazyLock};

use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};

use uuid::Uuid;
use zero2prod_axum::{
    configuration::{DatabaseSettings, get_configuration},
    startup::run,
    telemetry::init_telemetry,
};

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let subscriber_name = "test".into();
    let default_filter_level = "info".into();

    if env::var("TEST_LOG").is_ok() {
        init_telemetry(subscriber_name, default_filter_level, std::io::stdout);
    } else {
        init_telemetry(subscriber_name, default_filter_level, std::io::sink);
    }
});

pub struct TestAppState {
    address: String,
    pub pg_pool: PgPool,
}

async fn spawn_app() -> TestAppState {
    LazyLock::force(&TRACING);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind");

    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut config = get_configuration().expect("Failed to load configuration");
    config.database.database_name = Uuid::new_v4().to_string();

    let pg_pool = create_database(&config.database).await;

    let server = run(listener, pg_pool.clone());
    tokio::spawn(server);

    TestAppState { address, pg_pool }
}

pub async fn create_database(config: &DatabaseSettings) -> PgPool {
    // connect to superadmin
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: Secret::new("password".to_string()),
        ..config.clone()
    };

    let mut connection = PgConnection::connect(&maintenance_settings.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    // execute database connection using superadmin
    connection
        .execute(
            format!(
                r#"CREATE DATABASE "{}" WITH OWNER "{}";"#,
                &config.database_name, &config.username
            )
            .as_str(),
        )
        .await
        .expect("Failed to create database");

    // excute migrations
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to Migrate database");

    connection_pool
}

#[tokio::test]
async fn heath_check() {
    let state = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health-check", &state.address))
        .send()
        .await
        .expect("Health Check failed");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn test_valid_form_data() {
    let state = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let result = client
        .post(format!("{}/subscriptions", &state.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed execute request request");

    assert_eq!(200, result.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&state.pg_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn test_400_data_missing() {
    let state = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &state.address))
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
