use powerdown_wiki::{ServerState, initiate_scheduled_tasks, handle_shutdown_signal};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let state = ServerState::initalize().await;

    let app = powerdown_wiki::router(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    initiate_scheduled_tasks(state.clone());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(handle_shutdown_signal(state))
    .await
    .unwrap();
}
