use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::{Html, IntoResponse}, BoxError, Router
};
use axum_extra::routing::RouterExt;
use std::{sync::Arc, time};
use tower::{ServiceBuilder, layer::Layer};

use tower_http::normalize_path::NormalizePathLayer;

mod index;
mod static_files;
mod characters;
mod test_data;
mod utils;
mod errs;
mod stories;
mod user;

pub mod server_state;
pub use server_state::ServerState;

// TODO: Figure out how to pass around ServerState
pub fn router() -> Router {
    Router::new()
        .merge(index::router())
        .nest("/static", static_files::router())
        .nest("/characters", characters::router())
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