use super::structs;
use crate::{
    errs::RootErrors,
    user::{User, UsermadePost},
    utils::template_to_response,
    ServerState,
};
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

    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        None => return Err(RootErrors::Unauthorized),
        Some(requesting_user) => requesting_user,
    };

    let requested_art = match structs::PageArt::get_by_slug(&db_connection, &art_slug).await {
        None => {
            return Err(RootErrors::NotFound(
                original_uri,
                cookie_jar,
                Some(requesting_user),
            ))
        }
        Some(requested_art) => requested_art,
    };

    if !requested_art.can_be_modified_by(&requesting_user) {
        return Err(RootErrors::Forbidden);
    }

    // Remove the end bit, so it talks to /{art_slug} instead of /{art_slug}/edit
    let current_path = original_uri.path();
    let target_button_url = current_path[..current_path.rfind("/").unwrap()].to_string();

    Ok(template_to_response(super::post::ArtPostingPage {
        user: Some(requesting_user),
        original_uri,

        art_being_modified: Some(requested_art),
        target_button_url: Some(target_button_url),

        all_artist_names: super::get_all_artists(&db_connection).await,
    }))
}
