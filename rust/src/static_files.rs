
use axum::{routing::get_service, serve::Serve, Router};
use tower_http::services::ServeDir;

pub fn router() -> Router {
    Router::new().fallback_service(get_service(ServeDir::new("./static")))
}
