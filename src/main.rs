use sqlx::PgPool;
use zero2prod_axum::{configuration::get_configuration, startup::run, telemetry::init_telemetry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_telemetry("zero2prod".into(), "info".into(), std::io::stdout);

    let config = get_configuration().expect("Failed to load configuration");
    let address = format!("{}:{}", config.application.host, config.application.port);

    let pg_pool = PgPool::connect_lazy(&config.database.connection_string())
        .expect("Failed to create Postgres connection pool");

    let listener = tokio::net::TcpListener::bind(address).await?;
    run(listener, pg_pool).await?;

    Ok(())
}
