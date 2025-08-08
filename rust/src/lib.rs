use axum::{routing::get, Router};

mod index;
mod static_files;

pub fn router() -> Router {
    Router::new()
        .nest("/", index::router())
        .nest("/static/", static_files::router())
}