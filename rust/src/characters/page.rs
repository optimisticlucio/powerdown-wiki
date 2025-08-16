use axum::{extract::Path, response::{Html, IntoResponse}};
use askama::Template;
use super::structs::Character;
use crate::errs::{RootErrors};
use crate::test_data;


pub async fn character_page(
    Path(character_slug): Path<String>
) -> impl IntoResponse {
    // TODO: Actually connect to a database.
    if let Some(chosen_char) = test_data::get_test_characters().iter().find(|character| character.name.to_lowercase() == character_slug) {
        unimplemented!()
    }
    else {
        RootErrors::NOT_FOUND
    }
}