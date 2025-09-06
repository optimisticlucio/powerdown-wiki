use axum::{
    extract::State, response::Html, routing::get, Router};
use askama::{Template};
use rand::seq::IndexedRandom;
use crate::{user::User, test_data};
use lazy_static::lazy_static;
use crate::ServerState;

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

    FrontpageTemplate {
        user: None,
        buttons: &FRONTPAGE_ITEMS,
        random_quote: &random_subtitle,
        random_ad: &test_data::get_frontpage_ads().choose(&mut rand::rng()).unwrap(),
    }.render().unwrap().into()
}