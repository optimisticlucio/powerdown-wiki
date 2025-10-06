
use axum::{routing::get_service, Router};
use tower_http::services::ServeDir;
use crate::ServerState;

pub fn router() -> Router<ServerState> {
    Router::new().fallback_service(get_service(ServeDir::new("./static")))
}
