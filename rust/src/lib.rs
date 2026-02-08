use axum::{
    error_handling::HandleErrorLayer,
    extract::{OriginalUri, State},
    response::{IntoResponse, Redirect},
    routing::get,
    BoxError, Router,
};
use axum_extra::routing::RouterExt;
use http::Uri;
use std::time;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::compression::CompressionLayer;

mod art;
mod askama;
mod characters;
mod errs;
mod index;
mod misc;
mod server_state;
mod static_files;
mod stories;
mod test_data;
mod user;
mod utils;
mod scheduled_tasks;
mod graceful_shutdown;
mod admin;
mod refinery_migrations;
mod nsfw_splash;

pub use errs::RootErrors;
pub use server_state::ServerState;
pub use scheduled_tasks::initiate_scheduled_tasks;
pub use graceful_shutdown::handle_shutdown_signal;
pub use refinery_migrations::run_migrations;

use crate::user::User;

pub fn router(state: ServerState) -> Router<()> {
    Router::new()
        .merge(index::router())
        .nest("/static", static_files::router())
        .nest("/characters", characters::router())
        .nest("/art", art::router())
        .route_with_tsr(
            "/art-archive",
            get(|uri: Uri| async move { Redirect::permanent(&format!("/art{}", uri.path())) }),
        )
        .nest("/stories", stories::router())
        .nest("/user", user::router())
        .nest("/misc", misc::router())
        .nest("/admin", admin::router())

        .fallback(fallback)
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(root_error_handler))
                .timeout(time::Duration::from_secs(30))
                .layer(CookieManagerLayer::new())
                .layer(CompressionLayer::new()),
        )
}

async fn root_error_handler(err: BoxError) -> impl IntoResponse {
    if err.is::<tower::timeout::error::Elapsed>() {
        errs::RootErrors::RequestTimeout
    } else {
        errs::RootErrors::InternalServerError
    }
}

async fn fallback(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<RootErrors, RootErrors> {
    let requesting_user = User::easy_get_from_cookie_jar(&state, &cookie_jar).await?;
    Ok(errs::RootErrors::NotFound(
        original_uri,
        cookie_jar,
        requesting_user,
    ))
}
