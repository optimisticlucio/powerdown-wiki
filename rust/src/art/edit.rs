use super::structs;
use crate::{errs::RootErrors, user::User, utils::template_to_response, ServerState};
use axum::{
    extract::{OriginalUri, Path, State},
    response::Response,
};

pub async fn edit_art_page(
    Path(art_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();
    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    if let Some(requested_art) =
        structs::PageArt::get_by_slug(&state.db_pool.get().await.unwrap(), &art_slug).await
    {
        // Remove the end bit, so it talks to /{art_slug} instead of /{art_slug}/edit
        let current_path = original_uri.path();
        let target_button_url = current_path[..current_path.rfind("/").unwrap()].to_string();
        
        Ok(template_to_response(super::post::ArtPostingPage {
            user: requesting_user,
            original_uri,

            art_being_modified: Some(requested_art),
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
