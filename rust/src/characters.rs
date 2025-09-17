use askama::Template;
use axum::{response::Html, routing::get, extract::State, Router};
use crate::{user::User, test_data, utils, ServerState};
use chrono;
use axum_extra::routing::RouterExt;

mod page;
pub mod structs;

pub use structs::{BaseCharacter, PageCharacter};

pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(character_index))
        .route_with_tsr("/{character_slug}", get(page::character_page))
}

#[derive(Template)] 
#[template(path = "characters/index.html")]
struct CharacterIndex<'a> {
    user: Option<User>,
    random_subtitle: &'a str,
    characters: &'a Vec<BaseCharacter>,
    birthday_characters: &'a Vec<BaseCharacter>,
    birthday_character_names: &'a str,
    date_today_readable: &'a str,
}

async fn character_index(State(state): State<ServerState>) -> Html<String> {
    let displayed_characters: Vec<BaseCharacter> = BaseCharacter::get_all_characters(state.db_pool.get().await.unwrap())
                .await.into_iter()
                .filter(|base_character| !base_character.is_hidden && !base_character.is_archived)
                .collect();

    let current_time = chrono::Utc::now();
    let date_today_readable = utils::format_date_to_human_readable(current_time);

    let birthday_characters: Vec<BaseCharacter> = BaseCharacter::get_birthday_characters(state.db_pool.get().await.unwrap()).await;
    let birthday_character_names = { 
        let only_names: Vec<&str> = birthday_characters.iter().map(|x| x.name.as_str()).collect();
        utils::join_names_human_readable(only_names)
    };

    let random_subtitle = {
        let statement = "SELECT * FROM quote WHERE association = 'character_index' ORDER BY RANDOM() LIMIT 1;"; 

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