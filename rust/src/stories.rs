use axum::Router;
use axum_extra::routing::RouterExt;
use crate::ServerState;
use axum::routing::{get, post};

mod structs;
mod post;
mod page;

pub fn router() -> Router<ServerState> {
    Router::new().route_with_tsr("/new", post(post::add_story))
            .route_with_tsr("/{story_slug}", post(post::update_story))
}