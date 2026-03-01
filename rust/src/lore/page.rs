use super::structs::PageLore;
use crate::utils::template_to_response;
use crate::{RootErrors, ServerState, User};
use askama::Template;
use axum::{
    extract::{OriginalUri, Path, State},
    response::Response,
};
use http::Uri;

#[derive(Debug, Template)]
#[template(path = "lore/page.html")]
struct LorePage {
    user: Option<User>,
    original_uri: Uri,

    page_lore: PageLore,
}

#[axum::debug_handler]
pub async fn lore_page(
    Path(lore_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let requested_lore = match PageLore::get_from_slug(&db_connection, &lore_slug).await {
        None => {
            return Err(RootErrors::NotFound(
                original_uri,
                cookie_jar,
                requesting_user,
            ))
        }
        Some(page) => page,
    };

    Ok(template_to_response(LorePage {
        user: requesting_user,
        original_uri,

        page_lore: requested_lore,
    }))
}
