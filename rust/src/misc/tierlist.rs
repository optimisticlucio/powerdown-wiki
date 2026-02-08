use crate::{
    characters::BaseCharacter, utils::template_to_response, RootErrors, ServerState, User,
};
use askama::Template;
use axum::{
    extract::{OriginalUri, State},
    response::Response,
};
use http::Uri;

#[derive(Debug, Template)]
#[template(path = "misc/tierlist.html")]
struct TierlistPage {
    user: Option<User>,
    original_uri: Uri,

    all_base_characters: Vec<BaseCharacter>,
}

/// Shows the user a tierlist generator with all available base characters.
pub async fn tierlist(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_err| RootErrors::InternalServerError)?;

    let user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let all_base_characters = BaseCharacter::get_all_characters(&db_connection).await;

    Ok(template_to_response(TierlistPage {
        user,
        original_uri,

        all_base_characters,
    }))
}
