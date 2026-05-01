use super::structs;
use crate::{
    errs::RootErrors, lore::structs::PageLore, user::User, utils::template_to_response, ServerState,
};
use askama::Template;
use axum::{
    extract::{OriginalUri, Path, State},
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

#[derive(Debug, Template)]
#[template(path = "lore/new.html")]
struct NewLorePage {
    user: Option<User>,
    original_uri: Uri,

    lore_categories: Vec<structs::LoreCategory>,
    lore_being_modified: Option<PageLore>,
}

pub async fn new_lore_page(
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

    let lore_categories = structs::LoreCategory::get_all_categories(&db_connection).await;

    Ok(template_to_response(NewLorePage {
        user: Some(requesting_user),
        original_uri,

        lore_being_modified: None,
        lore_categories,
    }))
}

pub async fn edit_lore_page(
    Path(lore_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();
    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let Some(requested_lore) = structs::PageLore::get_from_slug(&db_connection, &lore_slug).await
    else {
        return Err(RootErrors::NotFound(
            original_uri,
            cookie_jar,
            requesting_user,
        ));
    };

    let Some(requesting_user) = requesting_user else {
        return Err(RootErrors::Unauthorized);
    };

    if !requesting_user.user_type.permissions().can_modify_lore {
        return Err(RootErrors::Forbidden);
    }

    let lore_categories = structs::LoreCategory::get_all_categories(&db_connection).await;

    Ok(template_to_response(NewLorePage {
        user: Some(requesting_user),
        original_uri,

        lore_being_modified: Some(requested_lore),
        lore_categories,
    }))
}
