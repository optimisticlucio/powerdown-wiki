use axum::{extract::Path, http::StatusCode, response::{Html, IntoResponse}};
use askama::Template;
use super::structs::Character;


pub async fn character_page(
    Path(character_slug): Path<String>
) -> Result<Html<String>, StatusCode> {
    // TODO: Actually connect to a database.
    Err(StatusCode::NOT_FOUND)
}