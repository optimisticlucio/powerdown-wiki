use axum::{response::IntoResponse, routing::get, Router};
use axum_extra::routing::RouterExt;
use crate::{ServerState, RootErrors};

mod oauth_callback_handling;
mod structs;

pub use structs::User;

pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(user_page))
}

/// Returns the user page. If the user is not logged in, shows login page.
pub async fn user_page() -> Result<impl IntoResponse, RootErrors> {
    Ok("TODO: Implement user page") // TODO
}

/// Returns page allowing user to login using oauth methods.
pub async fn login_page() -> Result<impl IntoResponse, RootErrors> {
    Ok("TODO: Implement login page") // TODO
}

/// Returns page letting user view their own account.
pub async fn logged_user_page() -> Result<impl IntoResponse, RootErrors> {
    Ok("TODO: Implement user page") // TODO
}