use axum::{routing::get, Router};

mod index;
mod static_files;
mod characters;

pub fn router() -> Router {
    Router::new()
        .nest("/", index::router())
        .nest("/static/", static_files::router())
        .nest("/characters/", characters::router())
}