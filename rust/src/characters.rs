use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/", get("NOT IMPLEMENTED"))
}

struct Character {
    pub is_hidden: bool,
    pub archival_reason: Option<String>, // If none, not archived.

    pub name: String,
    pub long_name: Option<String>,
    pub subtitles: Vec<String>,
    // TODO: character author
    // TODO: character logo
    // TODO: character birthday
    pub thumbnail_url: String,
    pub img_url: String,
    pub infobox: Vec<(String, String)>,
    // TODO: relationships?
    // TODO: custom css
}

// TODO: Get character ritual info
