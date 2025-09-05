use powerdown_wiki::ServerState;
use tower::Layer;
use axum::{
    ServiceExt, // for `into_make_service`
    extract::Request,
};
use tower_http::normalize_path::NormalizePathLayer;

#[tokio::main]
async fn main() {
    // TODO: Move the middleware soldering outside of main.
    let app = powerdown_wiki::router(ServerState::initalize().await);

    // this can be any `tower::Layer`
    let middleware = NormalizePathLayer::trim_trailing_slash();

    // apply the layer around the whole `Router`
    // this way the middleware will run before `Router` receives the request
    let app_with_middleware = middleware.layer(app);

    let listener = tokio::net::TcpListener::bind("localhost:8080").await.unwrap();

    axum::serve(listener, ServiceExt::<Request>::into_make_service(app_with_middleware))
        .await
        .unwrap();
}
