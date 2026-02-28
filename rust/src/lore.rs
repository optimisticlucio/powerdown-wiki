use crate::{ServerState, User, RootErrors};
use askama::Template;
use axum::{routing::get, Router, response::Response};
use axum_extra::routing::RouterExt;
use axum::extract::{State, OriginalUri};
use crate::utils::template_to_response;
use http::{Uri};

mod structs;
mod page;
mod modify;

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

    Ok(
        template_to_response(
            LoreIndex {
                user: requesting_user,
                original_uri,
            }
        )
    )
}

#[derive(Debug, Template)]
#[template(path = "lore/index.html")]
struct LoreIndex {
    user: Option<User>,
    original_uri: Uri,
}