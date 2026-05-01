use super::structs;
use crate::{errs::RootErrors, user::User, utils::template_to_response, ServerState};
use askama::Template;
use axum::{
    extract::{OriginalUri, State},
    response::Response,
};
use http::Uri;

#[derive(Debug, Template)]
#[template(path = "lore/edit-categories.html")]
struct EditCategoriesPage {
    user: Option<User>,
    original_uri: Uri,

    sorted_lore_categories: Vec<structs::LoreCategory>,
}

pub async fn edit_categories(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();
    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let Some(requesting_user) = requesting_user else {
        return Err(RootErrors::Unauthorized);
    };

    if !requesting_user.user_type.permissions().can_modify_lore {
        return Err(RootErrors::Forbidden);
    }

    let sorted_lore_categories = structs::LoreCategory::get_all_categories(&db_connection).await;

    Ok(template_to_response(EditCategoriesPage {
        user: Some(requesting_user),
        original_uri,

        sorted_lore_categories,
    }))
}
