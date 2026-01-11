use crate::ServerState;
use axum::Router;

pub fn router() -> Router<ServerState> {
    Router::new()
}
