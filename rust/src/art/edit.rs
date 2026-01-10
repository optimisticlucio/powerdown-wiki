use askama::Template;
use crate::{errs::RootErrors, user::User, ServerState, utils::template_to_response};
use http::{Uri};
use tower_cookies::Cookies;
use axum::{extract::{OriginalUri, Path, Query, State}, response::{IntoResponse, Response}};
use super::structs;

#[derive(Template)]
#[template(path = "art/edit.html")]
struct EditArtPage {
    user: Option<User>,
    original_uri: Uri,

    title: String,
}

pub async fn edit_art_page(
    Path(art_slug): Path<String>,
    State(state): State<ServerState>,
    Query(query_params): Query<structs::ArtSearchParameters>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    if let Some(requested_art) = structs::PageArt::get_by_slug(&state.db_pool.get().await.unwrap(), &art_slug).await {
        Ok(template_to_response(
        EditArtPage {
            user: User::get_from_cookie_jar(&db_connection, &cookie_jar).await,
            original_uri,

            title: requested_art.base_art.title
        })
        )
    }
    else {
        Err(RootErrors::NOT_FOUND(original_uri, cookie_jar))
    }

}
