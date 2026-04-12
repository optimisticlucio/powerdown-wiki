use crate::characters::BaseCharacter;
use crate::{RootErrors, ServerState};
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};

pub async fn random_character_redirect(
    State(state): State<ServerState>,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.map_err(|err| {
        eprintln!("[RANDOM CHARACTER REDIRECT] Failed getting the DB connection! Error: {err:?}");
        RootErrors::InternalServerError
    })?;

    let random_character = BaseCharacter::get_random_character(&db_connection).await;

    Ok(Redirect::to(&format!("/characters/{}", random_character.slug)).into_response())
}
