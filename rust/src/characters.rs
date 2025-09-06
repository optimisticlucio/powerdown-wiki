use askama::Template;
use axum::{response::Html, routing::get, extract::State, Router};
use rand::seq::IndexedRandom;
use crate::{user::User, test_data, utils, ServerState};
use lazy_static::lazy_static;
use chrono;
use axum_extra::routing::RouterExt;

mod page;
pub mod structs;

pub use structs::{Character, BaseCharacter, PageCharacter};

pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(character_index))
        .route_with_tsr("/{character_slug}", get(page::character_page))
}

pub fn get_with_birthday_today() -> Vec<Character> {
    unimplemented!("Return characters who's birthday is today, relative to the server.")
}

#[derive(Template)] 
#[template(path = "characters/index.html")]
struct CharacterIndex<'a> {
    user: Option<User>,
    random_subtitle: &'a str,
    characters: &'a Vec<Character>,
    birthday_characters: &'a Vec<Character>,
    birthday_character_names: &'a str,
    date_today_readable: &'a str,
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

async fn character_index(State(state): State<ServerState>) -> Html<String> {
    let test_characters = test_data::get_test_characters(); // TODO: Hook up to DB.

    let displayed_characters: Vec<Character> = test_characters.iter().filter(|character| !character.is_hidden && character.archival_reason.is_none()).map(Character::clone).collect();

    let current_time = chrono::Utc::now();
    let date_today_readable = utils::format_date_to_human_readable(current_time);

    let birthday_characters: Vec<Character> = vec![test_characters.get(2).unwrap().clone()]; // TODO: Actually check who's birthday it is.
    let birthday_character_names = { 
        let only_names: Vec<&str> = birthday_characters.iter().map(|x| x.name.as_str()).collect();
        utils::join_names_human_readable(only_names)
    };

    let random_subtitle = {
        let statement = "SELECT *  FROM quote WHERE association = 'character_index' ORDER BY RANDOM() LIMIT 1;"; 

        match state.db_pool.get().await {
            // TODO: Turn this unwrap into something that handles error better.
            Ok(db_connection) => 
                db_connection.query(statement, &[]).await.unwrap()
                    .get(0).unwrap()
                    .get(0),
            _ => "Insert funny text here.".to_owned() // "Oh shit it broke" text that won't seem too odd for a random user.
        }
    };

    CharacterIndex {
        user: None,
        random_subtitle: &random_subtitle,
        characters: &displayed_characters,
        birthday_characters: &birthday_characters,
        date_today_readable: &date_today_readable,
        birthday_character_names: &birthday_character_names
    }.render().unwrap().into()
}