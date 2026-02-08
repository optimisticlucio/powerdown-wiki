use askama::Template;
use axum::response::Response;
use http::Uri;

use crate::{User, utils::template_to_response};

#[derive(Debug, Template)]
#[template(path = "nsfw_splash.html")]
struct NSFWSplash<'a> {
    user: &'a Option<User>,
    original_uri: &'a Uri,
}

/// Reads the user's cookies. If the user doesn't have NSFW viewing enabled, returns the NSFW splash page.
pub fn get_if_user_hasnt_enabled_nsfw(
    user: &Option<User>,
    original_uri: &Uri,
    cookie_jar: &tower_cookies::Cookies
) -> Option<Response> {
    let user_has_enabled_nsfw = cookie_jar.get("NSFW_WARNING_SHOWN").is_some();

    if user_has_enabled_nsfw {
        None
    } else {
        Some(
            template_to_response(
                NSFWSplash {
                    user,
                    original_uri
                }
            )
        )
    }
}
