use axum::{
    routing::get,
    response::Html,
    Router};
use askama::{Template};
use crate::navbar::{self, Navbar};
use lazy_static::lazy_static;

pub fn router() -> Router {
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
            url: "https://powerdown.wiki/characters",
            image_url: "https://powerdown.wiki/assets/img/characters.png"
        }
    ];
}

#[derive(Template)] 
#[template(path = "index.html")]
struct FrontpageTemplate {
    title: &'static str,
    navbar: Navbar,
    buttons: &'static Vec<FrontpageItem>
}

async fn homepage() -> Html<String> {
    FrontpageTemplate {
        title: "Front Page",
        navbar: Navbar::not_logged_in(),
        buttons: &FRONTPAGE_ITEMS
    }.render().unwrap().into()
}