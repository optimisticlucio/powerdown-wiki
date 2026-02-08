use crate::{characters, ServerState};
use crate::{test_data, user::User, utils, RootErrors};
use askama::Template;
use axum::{
    extract::{OriginalUri, State},
    response::Response,
    routing::get,
    Router,
};
use axum_extra::routing::RouterExt;
use http::Uri;
use lazy_static::lazy_static;
use rand::seq::IndexedRandom;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(homepage))
        .route_with_tsr("/onboarding", get(onboarding))
}

#[derive(Debug)]
struct FrontpageItem {
    pub name: &'static str,
    pub url: &'static str,
    pub image_url: &'static str,
}

lazy_static! {
    static ref FRONTPAGE_ITEMS: Vec<FrontpageItem> = vec![
        FrontpageItem {
            name: "Art",
            url: "/art",
            image_url: "/static/img/frontpage/art.jpg"
        },
        FrontpageItem {
            name: "Characters",
            url: "/characters",
            image_url: "/static/img/frontpage/characters.png"
        },
        FrontpageItem {
            name: "Stories",
            url: "/stories",
            image_url: "/static/img/frontpage/stories.jpg"
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
    today_date: &'a str,
    discord_link: Option<String>,
}

async fn homepage(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::InternalServerError)?;

    let random_subtitle: String = {
        let statement =
            "SELECT * FROM quote WHERE association = 'homepage' ORDER BY RANDOM() LIMIT 1;";

        db_connection
            .query(statement, &[])
            .await
            .unwrap()
            .get(0)
            .unwrap()
            .get(0)
    };

    let user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let birthday_characters =
        characters::BaseCharacter::get_birthday_characters(&db_connection).await;

    let discord_link = utils::arbitrary_values::get_discord_link(&db_connection).await;

    Ok(utils::template_to_response(FrontpageTemplate {
        user,
        original_uri,

        buttons: &FRONTPAGE_ITEMS,
        random_quote: &random_subtitle,
        random_ad: &test_data::get_frontpage_ads()
            .choose(&mut rand::rng())
            .unwrap(),
        birthday_characters,
        today_date: &utils::format_date_to_human_readable(chrono::Local::now().into()),

        discord_link,
    }))
}

#[derive(Debug, Template)]
#[template(path = "onboarding.html")]
struct Onboarding {
    user: Option<User>,
    original_uri: Uri,

    discord_link: Option<String>,
}

async fn onboarding(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::InternalServerError)?;

    let user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let discord_link = utils::arbitrary_values::get_discord_link(&db_connection).await;

    Ok(utils::template_to_response(Onboarding {
        user,
        original_uri,

        discord_link,
    }))
}
