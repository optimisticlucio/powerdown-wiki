use super::structs;
use crate::{
    errs::RootErrors, misc::structs::MiscItem, user::User, utils::template_to_response, ServerState,
};
use askama::Template;
use axum::{
    extract::{OriginalUri, State},
    response::Response,
};
use http::Uri;

pub async fn edit_misc_listing(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    // Who's trying to do this?
    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        Some(user) => user,
        None => return Err(RootErrors::Unauthorized),
    };

    if !requesting_user.user_type.permissions().can_modify_misc {
        return Err(RootErrors::Forbidden);
    }

    let misc_items = MiscItem::get_all(&db_connection).await;

    Ok(template_to_response(MiscEditPage {
        user: Some(requesting_user),
        original_uri,

        misc_items,
    }))
}

#[derive(Debug, Template)]
#[template(path = "misc/edit.html")]
pub struct MiscEditPage {
    pub user: Option<User>,
    pub original_uri: Uri,

    pub misc_items: Vec<MiscItem>,
}
