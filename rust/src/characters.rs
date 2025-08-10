use std::clone;

use askama::Template;
use axum::{response::Html, routing::get, Router};
use rand::seq::IndexedRandom;
use crate::{navbar::Navbar, test_data};
use lazy_static::lazy_static;

pub fn router() -> Router {
    Router::new().route("/", get(character_index))
}

#[derive(Clone)]
pub struct Character {
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



pub fn get_with_birthday_today() -> Vec<Character> {
    unimplemented!("Return characters who's birthday is today, relative to the server.")
}

#[derive(Template)] 
#[template(path = "character-index.html")]
struct CharacterIndex<'a> {
    navbar: Navbar,
    random_subtitle: &'a str,
    characters: &'a Vec<Character>,
    birthday_characters: &'a Vec<Character>
}

lazy_static!{
    // TODO - Make this into a goddamn database.
    static ref RANDOM_SUBTITLES: Vec<String> = vec![
        "Everyone on this list have some sort of a police record. Especially the cops.",
        "Fun Fact: this project used to be a self-insert VN for a discord server.",
        "The fact we haven't been sued by the X-Men writers bewilders us.",
        "AKA The Children of Purity's hitlist."
        ].into_iter().map(String::from).collect();
}

async fn character_index() -> Html<String> {
    let test_characters = test_data::get_test_characters(); // TODO: Hook up to DB.

    let displayed_characters: Vec<Character> = test_characters.iter().filter(|character| !character.is_hidden && character.archival_reason.is_none()).map(Character::clone).collect();

    CharacterIndex {
        navbar: Navbar::not_logged_in(),
        random_subtitle: RANDOM_SUBTITLES.choose(&mut rand::rng()).unwrap(),
        characters: &displayed_characters,
        birthday_characters: &vec![test_characters.get(2).unwrap().clone()] // TODO: Actually check who's birthday it is.
    }.render().unwrap().into()
}