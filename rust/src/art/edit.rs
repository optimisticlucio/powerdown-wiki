use super::structs;
use crate::{errs::RootErrors, user::User, utils::template_to_response, ServerState};
use askama::Template;
use axum::{
    extract::{OriginalUri, Path, State},
    response::Response,
};
use http::Uri;

#[derive(Debug, Template)]
#[template(path = "art/edit.html")]
struct EditArtPage {
    user: Option<User>,
    original_uri: Uri,

    title: String,
}

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
        Ok(template_to_response(EditArtPage {
            user: requesting_user,
            original_uri,

            title: requested_art.base_art.title,
        }))
    } else {
        Err(RootErrors::NotFound(original_uri, cookie_jar, requesting_user))
    }
}
