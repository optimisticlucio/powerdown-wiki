use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::{Html, IntoResponse, Redirect}, routing::get, BoxError, Router
};
use axum_extra::routing::RouterExt;
use http::Uri;
use std::{sync::Arc, time};
use tower::{ServiceBuilder, layer::Layer};
use tower_cookies::{CookieManagerLayer};

mod index;
mod static_files;
mod characters;
mod test_data;
mod utils;
mod errs;
mod stories;
mod user;
mod art;
mod askama;
mod server_state;

pub use server_state::ServerState;
pub use errs::RootErrors;

pub fn router(state: ServerState) -> Router<()> {
    Router::new()
        .merge(index::router())
        .nest("/static", static_files::router())
        .nest("/characters", characters::router()) 
        .nest("/art", art::router())
        .route_with_tsr("/art-archive", get(|uri: Uri| async move { Redirect::permanent(&format!("/art{}", uri.path()))}))
        .layer(CookieManagerLayer::new())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(root_error_handler))
                .timeout(time::Duration::from_secs(15))
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