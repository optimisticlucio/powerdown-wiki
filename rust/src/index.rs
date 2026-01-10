use axum::{
    Router, extract::{OriginalUri, State}, response::{Html, Response}, routing::get};
use askama::{Template};
use http::Uri;
use rand::seq::IndexedRandom;
use crate::{RootErrors, test_data, user::User, utils};
use lazy_static::lazy_static;
use crate::{ServerState, characters};


pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(homepage))
}

#[derive(Debug)]
struct FrontpageItem {
    pub name: &'static str,
    pub url: &'static str,
    pub image_url: &'static str
}

lazy_static! {
    static ref FRONTPAGE_ITEMS: Vec<FrontpageItem> = vec![
        FrontpageItem {
            name: "Art",
            url: "/art",
            image_url: "https://powerdown.wiki/assets/img/art.png"
        },
        FrontpageItem {
            name: "Characters",
            url: "/characters",
            image_url: "https://powerdown.wiki/assets/img/characters.png"
        },
        FrontpageItem {
            name: "Stories",
            url: "/stories",
            image_url: "https://powerdown.wiki/assets/img/art-archive/thumbnails/master-tactics.png"
        }
    ];
}

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct FrontpageTemplate<'a> {
    user: Option<User>,
    original_uri: Uri,
    
    buttons: &'static Vec<FrontpageItem>,
    random_quote: &'a str,
    random_ad: &'a str,
    birthday_characters: Vec<characters::BaseCharacter>,
    today_date: &'a str
}

async fn homepage(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies
    ) -> Result<Response, RootErrors> {
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

    Ok(utils::template_to_response(FrontpageTemplate {
        user: User::easy_get_from_cookie_jar(&state, &cookie_jar).await?,
        original_uri,

        buttons: &FRONTPAGE_ITEMS,
        random_quote: &random_subtitle,
        random_ad: &test_data::get_frontpage_ads().choose(&mut rand::rng()).unwrap(),
        birthday_characters,
        today_date: &utils::format_date_to_human_readable(chrono::Local::now().into())
    }))
}