use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::Html, BoxError, Router
};
use std::time;
use tower::ServiceBuilder;
use std::fs;
use lazy_static::lazy_static;

mod index;
mod static_files;
mod characters;
mod navbar;

pub fn router() -> Router {
    Router::new()
        .merge(index::router())
        .nest("/static/", static_files::router())
        .nest("/characters/", characters::router())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(root_error_handler))
                .timeout(time::Duration::from_secs(10))
        )
        .fallback(page_not_found)
}

lazy_static! {
    static ref INTERNAL_SERVER_ERROR_PAGE_CONTENT: String = fs::read_to_string("static/500.html").unwrap_or(String::from("SHIT'S FUCKED. BOTH AN INTERNAL ERROR AND UNABLE TO READ THE 505 PAGE. PAGE LUCIO, STAT."));
}

async fn root_error_handler(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            INTERNAL_SERVER_ERROR_PAGE_CONTENT.clone()
        )
    }
}

async fn page_not_found() -> (StatusCode, Html<String>) {
    // TODO: Put navbar on this shit, it looks naked.
    (
        StatusCode::NOT_FOUND, 
        fs::read_to_string("static/404.html")
            .unwrap_or(String::from("404 PAGE CONTENT DID NOT LOAD. PAGE LUCIO, STAT.")).into()
    )
}