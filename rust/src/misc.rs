use axum::Router;
use crate::ServerState;


pub fn router() -> Router<ServerState> {
    Router::new()
}