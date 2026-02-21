use crate::{
    user::structs::UserSession,
    utils::{self, template_to_response},
    RootErrors, ServerState,
};
use askama::Template;
use axum::{
    extract::{OriginalUri, Path, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use axum_extra::routing::RouterExt;
use http::Uri;
use tower_cookies::{Cookie, Cookies};

mod oauth;
mod patch;
mod structs;
mod traits;

pub use structs::User;
pub use structs::UserType;
pub use traits::UsermadePost;

use structs::Oauth2Provider;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(self_user_page))
        .route_with_tsr("/login", get(login_page))
        .route_with_tsr("/logout", get(log_out))
        .nest("/oauth2", oauth::router())
        .route_with_tsr("/{user_id}", get(other_user_page).patch(patch::patch_user))
        .route_with_tsr("/{user_id}/modify", get(patch::modify_user_page))
}

/// Returns the user page. If the user is not logged in, redirects to login page.
pub async fn self_user_page(
    State(state): State<ServerState>,
    cookie_jar: Cookies,
) -> Result<Response, RootErrors> {
    let session_user = User::easy_get_from_cookie_jar(&state, &cookie_jar).await?;

    match session_user {
        // If they aren't logged in, we have no user data to show em. Toss towards the login page!
        None => Ok(Redirect::to("/user/login").into_response()),
        Some(viewed_user) => Ok(Redirect::to(&format!("/user/{}", viewed_user.id)).into_response()),
    }
}

#[derive(Debug, Template)]
#[template(path = "user/user_page.html")]
struct UserPageTemplate {
    user: Option<User>,
    original_uri: Uri,

    viewed_user: User,
}

/// Shows you the info on a given user
pub async fn other_user_page(
    Path(user_id): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_err| RootErrors::InternalServerError)?;

    let user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let parsed_user_id: i32 = match user_id.parse() {
        Ok(id) => id,
        Err(_err) => return Err(RootErrors::NotFound(original_uri, cookie_jar, user)),
    };

    let viewed_user = User::get_by_id(&db_connection, &parsed_user_id).await;

    match viewed_user {
        Some(viewed_user) => Ok(template_to_response(UserPageTemplate {
            user,
            original_uri,

            viewed_user,
        })),
        None => Err(RootErrors::NotFound(original_uri, cookie_jar, user)),
    }
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

/// Logs out the currently logged-in user.
pub async fn log_out(
    State(state): State<ServerState>,
    cookie_jar: Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_err| RootErrors::InternalServerError)?;

    // Is there a log-in cookie?
    let user_session_id = match cookie_jar.get("USER_SESSION_ID") {
        Some(user_session_cookie) => user_session_cookie.value().to_string(),
        None => return Err(RootErrors::Unauthorized),
    };

    // Cool. Regardless of what happens now, we do not want the user to have their cookie anymore.
    cookie_jar.remove(Cookie::new("USER_SESSION_ID", ""));

    // If None is returned here, it means the session doesn't exist. We don't need to clean it up.
    if let Some(user_session) = UserSession::get_by_id(&db_connection, &user_session_id).await {
        // The session does exist, we're gonna need to clean it up.

        let user_session_id = user_session.session_id.clone();
        let user_id = user_session.user.id;
        let user_display_name = user_session.user.display_name.clone();

        user_session.delete_from_db(&db_connection).await.map_err(|err| {
            eprintln!("[LOG OUT] Failed deleting user {user_display_name}'s (ID:{user_id}) user session (ID:{user_session_id}). ERR: {err:?}");
            RootErrors::InternalServerError
        })?;
    }

    Ok(Redirect::to("/").into_response())
}
