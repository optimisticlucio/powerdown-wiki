use crate::ServerState;
use axum::{routing::get_service, Router};
use tower_http::services::ServeDir;

pub fn router() -> Router<ServerState> {
    Router::new().fallback_service(get_service(ServeDir::new("./static")))
}
