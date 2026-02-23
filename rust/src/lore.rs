use crate::ServerState;
use axum::{routing::get, Router};
use axum_extra::routing::RouterExt;

mod structs;

pub fn router() -> Router<ServerState> {
    Router::new()
}
