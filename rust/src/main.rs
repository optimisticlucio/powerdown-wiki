use powerdown_wiki::ServerState;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = powerdown_wiki::router(ServerState::initalize().await);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
