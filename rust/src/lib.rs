use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::{Html, IntoResponse}, BoxError, Router
};
use axum_extra::routing::RouterExt;
use std::{sync::Arc, time};
use tower::{ServiceBuilder, layer::Layer};

mod index;
mod static_files;
mod characters;
mod test_data;
mod utils;
mod errs;
mod stories;
mod user;
mod art;

pub mod server_state;
pub use server_state::ServerState;

pub fn router(state: ServerState) -> Router<()> {
    Router::new()
        .merge(index::router())
        .nest("/static", static_files::router())
        .nest("/characters", characters::router())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(root_error_handler))
                .timeout(time::Duration::from_secs(10))
        )
        .fallback(fallback)
        .with_state(state)
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