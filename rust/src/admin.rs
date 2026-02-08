use crate::{user::UserType, utils::template_to_response, RootErrors, ServerState, User};
use askama::Template;
use axum::{
    extract::{OriginalUri, State},
    response::Response,
    routing::get,
    Router,
};
use axum_extra::routing::RouterExt;
use http::Uri;

mod arbitrary_values;

pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(admin_panel)).route_with_tsr(
        "/arbitrary_values",
        get(arbitrary_values::arbitrary_value_panel).patch(arbitrary_values::patch_arbitrary_value),
    )
}

#[derive(Debug, Template)]
#[template(path = "admin/index.html")]
struct AdminPanel {
    user: Option<User>,
    original_uri: Uri,
}

/// If an admin is logged in, shows the admin panel
pub async fn admin_panel(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_err| RootErrors::InternalServerError)?;

    let user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    if !user_is_admin(&user) {
        return Err(RootErrors::NotFound(original_uri, cookie_jar, user));
    }

    Ok(template_to_response(AdminPanel { user, original_uri }))
}

/// Given an Option<user>, returns whether the user inside it is an admin or not.
fn user_is_admin(user: &Option<User>) -> bool {
    user.as_ref()
        .is_some_and(|user| [UserType::Admin, UserType::Superadmin].contains(&user.user_type))
}
