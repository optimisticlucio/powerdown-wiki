use axum::{
    Router, extract::{OriginalUri, State}, response::{Html, IntoResponse, Redirect, Response}, routing::get};
use axum_extra::routing::RouterExt;
use crate::{RootErrors, ServerState, utils::{self, template_to_response}};
use askama::{Template};
use http::Uri;
use tower_cookies::Cookies;

mod oauth;
mod structs;

pub use structs::User;

use structs::Oauth2Provider;

pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(user_page))
        .route_with_tsr("/login", get(login_page))
        .nest("/oauth2", oauth::router())
}

/// Returns the user page. If the user is not logged in, redirects to login page.
pub async fn user_page(
        State(state): State<ServerState>,
        OriginalUri(original_uri): OriginalUri,
        cookie_jar: Cookies
    ) -> Result<Response, RootErrors> {

    let session_user = User::easy_get_from_cookie_jar(&state, &cookie_jar).await?;

    // If they aren't logged in, we have no user data to show em. Toss towards the login page!
    if session_user.is_none() {
        return Ok(Redirect::to("/user/login").into_response());
    }

    let session_user = session_user.unwrap();

    Ok(
        template_to_response(
            UserPageTemplate {
                user: Some(session_user),
                original_uri
            }
        )
    )
}

#[derive(Debug, Template)]
#[template(path = "user/user_page.html")]
struct UserPageTemplate {
    user: Option<User>,
    original_uri: Uri,
}

/// Returns page allowing user to login/create an account/connect an existing account using oauth methods.
pub async fn login_page(
        State(state): State<ServerState>,
        OriginalUri(original_uri): OriginalUri,
        cookie_jar: Cookies,
    ) -> Result<Response, RootErrors> {
    Ok(utils::template_to_response(LoginTemplate {
        user: User::easy_get_from_cookie_jar(&state, &cookie_jar).await?, // TODO: Check if user is logged in, and if so, connect more accounts!
        original_uri,

        discord_oauth_url: &Oauth2Provider::Discord.get_user_login_url(),
        google_oauth_url: &Oauth2Provider::Google.get_user_login_url(),
        github_oauth_url: &Oauth2Provider::Github.get_user_login_url(),
    }))
}

#[derive(Debug, Template)]
#[template(path = "user/login.html")]
struct LoginTemplate<'a> {
    user: Option<User>,
    original_uri: Uri,
    
    discord_oauth_url: &'a str,
    google_oauth_url: &'a str,
    github_oauth_url: &'a str,
}