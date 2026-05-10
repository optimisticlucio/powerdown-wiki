use crate::utils::template_to_response;
use crate::{RootErrors, ServerState, User};
use askama::Template;
use axum::extract::{OriginalUri, State};
use axum::{
    response::Response,
    routing::{get, post},
    Router,
};
use axum_extra::routing::RouterExt;
use http::Uri;

mod edit;
mod post;
mod structs;
mod tierlist;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(index))
        .route_with_tsr(
            "/edit",
            get(edit::edit_misc_listing).post(post::edit_misc_section),
        )
        .route_with_tsr("/tierlist", get(tierlist::tierlist))
}

#[axum::debug_handler]
pub async fn index(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let misc_items = structs::MiscItem::get_all(&db_connection).await;

    Ok(template_to_response(MiscIndex {
        show_edit_button: requesting_user
            .as_ref()
            .is_some_and(|user| user.user_type.permissions().can_modify_misc),

        user: requesting_user,
        original_uri,

        misc_items,
    }))
}

#[derive(Debug, Template)]
#[template(path = "misc/index.html")]
struct MiscIndex {
    user: Option<User>,
    original_uri: Uri,

    misc_items: Vec<structs::MiscItem>,

    show_edit_button: bool,
}
