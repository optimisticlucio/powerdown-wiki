use axum::{
    extract::State, response::Html, routing::get, Router};
use askama::{Template};
use rand::seq::IndexedRandom;
use crate::{test_data, user::User, utils};
use lazy_static::lazy_static;
use crate::{ServerState, characters};


pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(homepage))
}

struct FrontpageItem {
    pub name: &'static str,
    pub url: &'static str,
    pub image_url: &'static str
}

lazy_static! {
    static ref FRONTPAGE_ITEMS: Vec<FrontpageItem> = vec![
        FrontpageItem {
            name: "Art",
            url: "https://powerdown.wiki/art-archive",
            image_url: "https://powerdown.wiki/assets/img/art.png"
        },
        FrontpageItem {
            name: "Characters",
            url: "/characters",
            image_url: "https://powerdown.wiki/assets/img/characters.png"
        }
    ];
}

#[derive(Template)] 
#[template(path = "index.html")]
struct FrontpageTemplate<'a> {
    user: Option<User>,
    buttons: &'static Vec<FrontpageItem>,
    random_quote: &'a str,
    random_ad: &'a str,
    birthday_characters: Vec<characters::BaseCharacter>,
    today_date: &'a str
}

async fn homepage(State(state): State<ServerState>) -> Html<String> {
    let random_subtitle = {
        let statement = "SELECT *  FROM quote WHERE association = 'homepage' ORDER BY RANDOM() LIMIT 1;"; 

        match state.db_pool.get().await {
            // TODO: Turn this unwrap into something that handles error better.
            Ok(db_connection) => 
                db_connection.query(statement, &[]).await.unwrap()
                    .get(0).unwrap()
                    .get(0),
            _ => "Designed so well, that you're already seeing error texts on the home page. Message Lucio, something broke.".to_owned() // "Oh shit it broke" text that won't seem too odd for a random user.
        }
    };

    let birthday_characters = characters::BaseCharacter::get_birthday_characters(state.db_pool.get().await.unwrap()).await;

    FrontpageTemplate {
        user: None,
        buttons: &FRONTPAGE_ITEMS,
        random_quote: &random_subtitle,
        random_ad: &test_data::get_frontpage_ads().choose(&mut rand::rng()).unwrap(),
        birthday_characters,
        today_date: &utils::format_date_to_human_readable(chrono::Local::now().into())
    }.render().unwrap().into()
}