use crate::{utils::template_to_response, RootErrors, ServerState, User};
use askama::Template;
use axum::{
    extract::{OriginalUri, State},
    response::Response,
    routing::get,
    Router,
};
use axum_extra::routing::RouterExt;
use http::Uri;
use structs::RitualCharacterData;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(index))
        .route_with_tsr("inprogress", get(page::ritual_page))
        .route_with_tsr("complete", get(page::ritual_done))
}

mod edit;
mod page;
mod structs;

const AVERAGE_MINUTES_PER_CHARACTER: f64 = 2.0;

#[axum::debug_handler]
pub async fn index(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let all_characters_in_ritual = RitualCharacterData::get_all_ritualists(&db_connection)
        .await
        .map_err(|err| {
            eprintln!("[RITUAL INDEX] Failed to read ritualist data! {err:?}");
            RootErrors::InternalServerError
        })?;

    let amount_of_main_characters: f64 = all_characters_in_ritual
        .iter()
        .filter(|character| character.page_character.base_character.is_main_character)
        // safety: in prod this number isn't gonna cross 50, we're fine.
        .count() as f64;

    let amount_of_characters_in_general: f64 = all_characters_in_ritual
        // safety: in prod this number isn't gonna cross 150, we're fine.
        .len() as f64;

    let standard_length =
        minutes_to_human_readable_time(AVERAGE_MINUTES_PER_CHARACTER * amount_of_main_characters);

    let long_length = minutes_to_human_readable_time(
        AVERAGE_MINUTES_PER_CHARACTER * amount_of_characters_in_general,
    );

    Ok(template_to_response(RitualIndex {
        user: requesting_user,
        original_uri,

        standard_length,
        long_length,
    }))
}

#[derive(Debug, Template)]
#[template(path = "misc/ritual/index.html")]
struct RitualIndex {
    user: Option<User>,
    original_uri: Uri,

    /// The length to be listed for how long the standard ritual is.
    standard_length: String,

    /// The length to be listed for how long the FULL ritual is. In capslock!
    long_length: String,
}

fn minutes_to_human_readable_time(minutes: f64) -> String {
    let hours: u8 = (minutes / 60.0).floor() as u8;

    let minutes: u8 = (minutes % 60.0).ceil() as u8;

    let mut returned_string: String = String::new();

    if hours > 0 {
        returned_string.push_str(&format!("{hours} hours"));
    }

    if minutes > 0 {
        if !returned_string.is_empty() {
            returned_string.push_str(" and ");
        }

        returned_string.push_str(&format!("{minutes} minutes"));
    }

    returned_string
}
