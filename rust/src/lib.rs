use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::{Html, IntoResponse}, BoxError, Router
};
use axum_extra::routing::RouterExt;
use std::time;
use tower::{ServiceBuilder, layer::Layer};

use crate::navbar::Navbar;
use tower_http::normalize_path::NormalizePathLayer;

mod index;
mod static_files;
mod characters;
mod navbar;
mod test_data;
mod utils;
mod errs;

pub fn router() -> Router {
    Router::new()
        .merge(index::router())
        // Use nest_service, and not nest. The latter has some funky behaviour wrt trailing slashes.
        .nest_service("/static", static_files::router())
        .nest_service("/characters", characters::router())
        .layer(
            ServiceBuilder::new()
                .layer(NormalizePathLayer::trim_trailing_slash())
                .layer(HandleErrorLayer::new(root_error_handler))
                .timeout(time::Duration::from_secs(10))
        )
        .fallback(fallback)
}

async fn root_error_handler(err: BoxError) -> impl IntoResponse {
    if err.is::<tower::timeout::error::Elapsed>() {
        errs::RootErrors::REQUEST_TIMEOUT
    } else {
        errs::RootErrors::INTERNAL_SERVER_ERROR
    }
}

async fn fallback() -> impl IntoResponse {
    errs::RootErrors::NOT_FOUND
}