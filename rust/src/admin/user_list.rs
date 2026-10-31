use crate::{utils::template_to_response, RootErrors, ServerState, User};
use askama::Template;
use axum::{
    extract::{OriginalUri, State},
    response::Response,
};
use http::Uri;

/// Shows admins the entire list of users in the DB.
pub async fn user_list(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_err| RootErrors::InternalServerError)?;

    let user = User::login_with_cookie_jar(&db_connection, &cookie_jar).await;

    if !super::user_is_admin(&user) {
        return Err(RootErrors::NotFound(Box::new((
            original_uri,
            cookie_jar,
            user,
        ))));
    }

    let mut all_users = User::get_all_users(&db_connection).await.map_err(|err| {
        eprintln!("[USER LIST] Error in getting all users: {err:?}");
        RootErrors::InternalServerError
    })?;

    // For convenience of reading (and likely relevance), sort by user type, then by last seen.
    // Lastly by ID just so we have *something* that's consistent incase two users are last seen together.

    all_users.sort_by(|user1, user2| {
        user1
            .user_type
            .cmp(&user2.user_type)
            .reverse() // Descending "priority".
            .then_with(|| user1.last_seen.cmp(&user2.last_seen).reverse()) // Descending times last seen.
            .then_with(|| user1.id.cmp(&user2.id))
    });

    Ok(template_to_response(UserListPage {
        user,
        original_uri,

        all_users,
    }))
}

#[derive(Debug, Template)]
#[template(path = "admin/user_list.html")]
struct UserListPage {
    user: Option<User>,
    original_uri: Uri,

    all_users: Vec<User>,
}
