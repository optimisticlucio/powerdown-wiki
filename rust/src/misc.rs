use crate::ServerState;
use axum::{routing::get, Router};
use axum_extra::routing::RouterExt;

mod tierlist;

pub fn router() -> Router<ServerState> {
    Router::new().route_with_tsr("/tierlist", get(tierlist::tierlist))
}
