use axum::{
    routing::get,
    response::Html,
    Router};
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

async fn homepage() -> Html<String> {
    FrontpageTemplate {
        user: None,
        buttons: &FRONTPAGE_ITEMS,
        random_quote: &test_data::get_frontpage_quotes().choose(&mut rand::rng()).unwrap(),
        random_ad: &test_data::get_frontpage_ads().choose(&mut rand::rng()).unwrap(),
    }.render().unwrap().into()
}