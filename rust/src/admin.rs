use crate::{ServerState, RootErrors};
use axum::{
    extract::{DefaultBodyLimit, OriginalUri, Query, State},
    response::Response,
    routing::{get, post},
    Router,
};

pub fn router() -> Router<ServerState> {
    Router::new()
}