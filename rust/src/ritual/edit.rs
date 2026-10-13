use super::RitualCharacterData;
use crate::{
    characters::BaseCharacter, utils::template_to_response, RootErrors, ServerState, User,
};
use askama::Template;
use axum::{
    extract::{OriginalUri, Query, State},
    response::Response,
};
use http::Uri;

#[axum::debug_handler]
pub async fn edit_ritual_data_page(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let Some(requesting_user) = requesting_user else {
        return Err(RootErrors::Unauthorized);
    };

    if !requesting_user.user_type.permissions().can_update_ritual {
        return Err(RootErrors::Forbidden);
    };

    let all_characters = BaseCharacter::get_all_characters(&db_connection)
        .await
        .map_err(|err| {
            eprintln!("[RITUAL EDIT PAGE] Failed to read character data! {err:?}");
            RootErrors::InternalServerError
        })?;

    let ritualist_data = RitualCharacterData::get_all_ritualists(&db_connection)
        .await
        .map_err(|err| {
            eprintln!("[RITUAL EDIT PAGE] Failed to read ritualist data! {err:?}");
            RootErrors::InternalServerError
        })?;

    let base_characters_in_ritual: Vec<&BaseCharacter> = ritualist_data
        .iter()
        .map(|ritual_data| &ritual_data.page_character.base_character)
        .collect();

    let base_characters_without_ritual_data: Vec<&BaseCharacter> = all_characters
        .iter()
        .filter(|character| !base_characters_in_ritual.contains(character))
        .collect();

    todo!()
}
