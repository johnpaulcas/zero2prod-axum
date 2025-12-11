use zero2prod_axum::{configuration::get_configuration, run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_configuration().expect("Failed to load configuration");

    let address = format!("127.0.0.1:{}", config.application_port);

    let listener = tokio::net::TcpListener::bind(address).await?;
    run(listener).await
}
