use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/", get(homepage))
}

struct FrontpageItem {
    pub name: String,
    pub url: String,
    pub image_url: String
}

async fn homepage() -> &'static str {
    "Huh, it worked!"
}