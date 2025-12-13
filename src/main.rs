use sqlx::PgPool;
use zero2prod_axum::{configuration::get_configuration, startup::run, telemetry::init_telemetry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_telemetry("zero2prod".into(), "info".into());

    let config = get_configuration().expect("Failed to load configuration");
    let address = format!("127.0.0.1:{}", config.application_port);

    let pg_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed connecting to database");

    let listener = tokio::net::TcpListener::bind(address).await?;
    run(listener, pg_pool).await?;

    Ok(())
}
