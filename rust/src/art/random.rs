use crate::art::structs::BaseArt;
use crate::{RootErrors, ServerState};
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};

pub async fn random_art_redirect(State(state): State<ServerState>) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.map_err(|err| {
        eprintln!("[RANDOM ART REDIRECT] Failed getting the DB connection! Error: {err:?}");
        RootErrors::InternalServerError
    })?;

    let random_art = BaseArt::get_random_art(&db_connection).await;

    Ok(Redirect::to(&format!("/art/{}", random_art.slug)).into_response())
}
