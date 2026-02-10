use super::structs;
use crate::{errs::RootErrors, user::User, utils::template_to_response, ServerState};
use axum::{
    extract::{OriginalUri, Path, State},
    response::Response,
};

pub async fn edit_character_page(
    Path(character_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();
    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    if let Some(requested_character) =
        structs::PageCharacter::get_by_slug(&db_connection, &character_slug).await
    {
        // Remove the end bit, so it talks to /{character_slug} instead of /{character_slug}/edit
        let current_path = original_uri.path();
        let target_button_url = current_path[..current_path.rfind("/").unwrap()].to_string();

        Ok(template_to_response(super::post::CharacterPostingPage {
            user: requesting_user,
            original_uri,

            character_being_modified: Some(requested_character),
            target_button_url: Some(target_button_url),
        }))
    } else {
        Err(RootErrors::NotFound(
            original_uri,
            cookie_jar,
            requesting_user,
        ))
    }
}
