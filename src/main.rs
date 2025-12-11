use zero2prod_axum::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    run(listener).await
}
