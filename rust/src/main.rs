#![deny(missing_debug_impl)]

use axum::{
    extract::Request,
    ServiceExt, // for `into_make_service`
};
use powerdown_wiki::ServerState;
use std::net::SocketAddr;
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;

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
