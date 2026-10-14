use super::RitualCharacterData;
use crate::{utils::template_to_response, RootErrors, ServerState, User};
use askama::Template;
use axum::{
    extract::{OriginalUri, Query, State},
    response::Response,
};
use http::Uri;
use serde::Deserialize;

#[axum::debug_handler]
pub async fn ritual_page(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    Query(settings): Query<RitualSettings>,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let all_characters_in_ritual = RitualCharacterData::get_all_ritualists(&db_connection)
        .await
        .map_err(|err| {
            eprintln!("[RITUAL PAGE] Failed to read ritualist data! {err:?}");
            RootErrors::InternalServerError
        })?;

    let ritual_characters = match settings.version {
        RitualVersion::Standard => all_characters_in_ritual
            .into_iter()
            .filter(|character| character.page_character.base_character.is_main_character)
            .collect(),
        RitualVersion::Full => all_characters_in_ritual,
    };

    Ok(template_to_response(RitualInProgress {
        user: requesting_user,
        original_uri,

        ritual_characters,
    }))
}

#[derive(Debug, Template)]
#[template(path = "misc/ritual/inprogress.html")]
struct RitualInProgress {
    user: Option<User>,
    original_uri: Uri,

    ritual_characters: Vec<RitualCharacterData>,
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct RitualSettings {
    #[serde(default)]
    pub version: RitualVersion,
}

#[derive(Default, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RitualVersion {
    #[default]
    Standard,
    Full,
}

#[axum::debug_handler]
pub async fn ritual_done(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    Ok(template_to_response(RitualComplete {
        user: requesting_user,
        original_uri,
    }))
}

#[derive(Debug, Template)]
#[template(path = "misc/ritual/complete.html")]
struct RitualComplete {
    user: Option<User>,
    original_uri: Uri,
}
