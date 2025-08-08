use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = powerdown_wiki::router();
    let listener = tokio::net::TcpListener::bind("localhost:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
