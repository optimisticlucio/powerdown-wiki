use axum::Router;
use axum_extra::routing::RouterExt;
use crate::ServerState;
use axum::routing::{get, post};

mod structs;
mod post;

pub fn router() -> Router<ServerState> {
    Router::new().route_with_tsr("/new", post(post::add_story))
}