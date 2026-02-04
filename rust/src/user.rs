use crate::{
    utils::{self, template_to_response},
    RootErrors, ServerState,
};
use askama::Template;
use axum::{
    Router, extract::{OriginalUri, Path, State}, response::{IntoResponse, Redirect, Response}, routing::get
};
use axum_extra::routing::RouterExt;
use http::Uri;
use tower_cookies::Cookies;

mod oauth;
mod structs;
mod traits;
mod patch;

pub use structs::User;
pub use traits::UsermadePost;

use structs::Oauth2Provider;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(self_user_page))
        .route_with_tsr("/login", get(login_page))
        .nest("/oauth2", oauth::router())
        .route_with_tsr("/{user_id}", get(other_user_page).patch(patch::patch_user))
}

/// Returns the user page. If the user is not logged in, redirects to login page.
pub async fn self_user_page(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: Cookies,
) -> Result<Response, RootErrors> {
    let session_user = User::easy_get_from_cookie_jar(&state, &cookie_jar).await?;

    // If they aren't logged in, we have no user data to show em. Toss towards the login page!
    if session_user.is_none() {
        return Ok(Redirect::to("/user/login").into_response());
    }

    Ok(template_to_response(UserPageTemplate {
        user: session_user.clone(),
        original_uri,

        viewed_user: session_user,
    }))
}

#[derive(Debug, Template)]
#[template(path = "user/user_page.html")]
struct UserPageTemplate {
    user: Option<User>,
    original_uri: Uri,

    viewed_user: Option<User>,
}

/// Shows you the info on a given user
pub async fn other_user_page(
    Path(user_id): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await
        .map_err(|_err| {
            RootErrors::InternalServerError
        })?;

    let user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let parsed_user_id: i32 = match user_id.parse() {
        Ok(id) => id,
        Err(_err) => return Err(RootErrors::NotFound(original_uri, cookie_jar, user))
    };

    let viewed_user = User::get_by_id(
            &db_connection, 
            &parsed_user_id
        ).await;

    Ok(template_to_response(UserPageTemplate {
        user,
        original_uri,

        viewed_user,
    }))
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
