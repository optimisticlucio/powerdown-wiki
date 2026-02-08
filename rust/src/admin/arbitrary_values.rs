use crate::{
    utils::{self, template_to_response},
    RootErrors, ServerState, User,
};
use askama::Template;
use axum::{
    extract::{OriginalUri, State},
    response::{IntoResponse, Response},
    Json,
};
use http::{StatusCode, Uri};
use serde::Deserialize;

#[derive(Debug, Template)]
#[template(path = "admin/arbitrary_values.html")]
struct ArbitraryValuePanel {
    user: Option<User>,
    original_uri: Uri,

    current_discord_invite_url: Option<String>,
}

/// If an admin is logged in, shows the admin panel
pub async fn arbitrary_value_panel(
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

    if !super::user_is_admin(&user) {
        return Err(RootErrors::NotFound(original_uri, cookie_jar, user));
    }

    let current_discord_invite_url =
        utils::arbitrary_values::get_discord_link(&db_connection).await;

    Ok(template_to_response(ArbitraryValuePanel {
        user,
        original_uri,
        current_discord_invite_url,
    }))
}

#[derive(Debug, Deserialize)]
/// The input from the user for which value to change to what.
pub struct ArbitraryValueChange {
    arbitrary_value: String,
    set_to: String,
}

pub async fn patch_arbitrary_value(
    State(state): State<ServerState>,
    cookie_jar: tower_cookies::Cookies,
    Json(change_request): Json<ArbitraryValueChange>,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_err| RootErrors::InternalServerError)?;

    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let requesting_admin = match requesting_user {
        Some(_) => {
            if !super::user_is_admin(&requesting_user) {
                return Err(RootErrors::Forbidden);
            } else {
                requesting_user.unwrap()
            }
        }
        None => {
            return Err(RootErrors::Unauthorized);
        }
    };

    match change_request.arbitrary_value.as_str() {
        "discord_invite_url" => {
            utils::arbitrary_values::set_discord_link(&db_connection, &change_request.set_to).await
                .map_err(|err| {
                    eprintln!("[PATCH ARBITRARY VALUE] Changing value of Discord Invite URL to {} by admin {} failed! {:?}", change_request.set_to, requesting_admin.display_name, err);
                    RootErrors::InternalServerError
                })?;
        }
        _ => {
            return Err(RootErrors::BadRequest(
                "Invalid arbitrary value given".to_string(),
            ))
        }
    }

    Ok((StatusCode::OK).into_response())
}
