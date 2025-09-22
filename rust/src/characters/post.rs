use axum::extract::{multipart, Multipart, State};
use crate::ServerState;

pub async fn add_character(State(state): State<ServerState>, mut multipart: Multipart) {
    unimplemented!()
}