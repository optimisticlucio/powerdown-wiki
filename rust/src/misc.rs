use crate::ServerState;
use axum::{Router, routing::get};
use axum_extra::routing::RouterExt;

mod tierlist;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route_with_tsr("/tierlist", get(tierlist::tierlist))
}
