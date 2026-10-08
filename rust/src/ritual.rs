use crate::{utils::template_to_response, RootErrors, ServerState, User};
use askama::Template;
use axum::{
    extract::{OriginalUri, State},
    response::Response,
    routing::get,
    Router,
};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use http::Uri;

pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(index))
}

const AVERAGE_MINUTES_PER_CHARACTER: f64 = 2.0;

#[axum::debug_handler]
pub async fn index(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let amount_of_main_characters: f64 = 23.0; // TODO

    let amount_of_characters_in_general: f64 = 88.0; // TODO

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

struct RitualCharacterData {
    name: String,
    power_name: String,
    power_description: String,
}

impl RitualCharacterData {
    /// Gets RitualCharacterData for all characters in the database.
    pub async fn get_all_characters(db_connection: &Object<Manager>) -> Vec<Self> {
        // TODO: Select only what's necessary to speed it up.
        let character_rows = db_connection
            .query(
                "SELECT * FROM ritual_info INNER JOIN character ON ritual_info.character_id == character.id ORDER_BY character.short_name",
                &[],
            )
            .await
            .unwrap();

        let converted_rows: Result<Vec<Self>, _> =
            character_rows.iter().map(Self::try_from).collect();

        converted_rows
            .map_err(|err| eprintln!("Failed to read RitualCharacterData: {err:?}"))
            .unwrap()
    }
}

impl TryFrom<&tokio_postgres::Row> for RitualCharacterData {
    type Error = tokio_postgres::Error;

    fn try_from(row: &tokio_postgres::Row) -> Result<Self, Self::Error> {
        let long_name: Option<String> = row.try_get("long_name")?;

        let name = long_name.unwrap_or(row.try_get("short_name")?);

        Ok(Self {
            name,
            power_name: row.try_get("power_name")?,
            power_description: row.try_get("power_description")?,
        })
    }
}
