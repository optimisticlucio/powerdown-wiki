use crate::lore::structs::BaseLore;
use crate::utils::template_to_response;
use crate::{lore::structs::LoreCategory, RootErrors, ServerState, User};
use askama::Template;
use axum::extract::{OriginalUri, State};
use axum::{response::Response, routing::get, Router};
use axum_extra::routing::RouterExt;
use http::Uri;

mod modify;
mod page;
mod structs;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(index))
        .route_with_tsr("/{lore_slug}", get(page::lore_page))
}

#[axum::debug_handler]
pub async fn index(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    // This is an unholy mess. I apologize to whoever needs to fix this later, likely me.
    // We just need to get all of the categories and their subpages.

    let mut lore_categories = LoreCategory::get_all_categories(&db_connection).await;

    lore_categories.sort();

    let mut lore_categories_with_pages = Vec::new();

    for lore_category in lore_categories {
        let lore_category_pages = lore_category
            .get_associated_lore_bases(&db_connection)
            .await;
        lore_categories_with_pages.push((lore_category, lore_category_pages));
    }

    Ok(template_to_response(LoreIndex {
        show_uploader_bar: requesting_user
            .as_ref()
            .is_some_and(|user| user.user_type.permissions().can_modify_lore),

        user: requesting_user,
        original_uri,

        lore_categories_with_pages,
    }))
}

#[derive(Debug, Template)]
#[template(path = "lore/index.html")]
struct LoreIndex {
    user: Option<User>,
    original_uri: Uri,

    lore_categories_with_pages: Vec<(LoreCategory, Vec<BaseLore>)>,

    show_uploader_bar: bool,
}
