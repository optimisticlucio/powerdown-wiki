use crate::user::User;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use http::Uri;
use lazy_static::lazy_static;
use std::fs;
use tower_cookies::Cookies;

#[derive(Debug)]
pub enum RootErrors {
    /// User asked for something that the server doesn't recognize.
    NotFound(Uri, Cookies),
    /// Part of my code ate shit and it's not the user's fault.
    InternalServerError,
    /// My code took too long to respond.
    RequestTimeout,
    /// Part of my code ate shit and it *is* the user's fault.
    BadRequest(Uri, Cookies, String),
    /// The user tried doing an action requiring to be logged in, and they aren't.
    Unauthorized,
    /// The user is logged in, and they don't have the permissions to do what they were doing.
    Forbidden,
}

impl IntoResponse for RootErrors {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(original_uri, cookie_jar) => page_not_found().into_response(),
            Self::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html::from(INTERNAL_SERVER_ERROR_PAGE_CONTENT.clone()),
            )
                .into_response(),
            Self::RequestTimeout => request_timeout().into_response(),
            Self::BadRequest(original_uri, cookie_jar, elaboration) => {
                bad_request(elaboration).into_response()
            }
            Self::Unauthorized => unauthorized().into_response(),
            Self::Forbidden => forbidden().into_response(),
        }
    }
}

lazy_static! {
    static ref INTERNAL_SERVER_ERROR_PAGE_CONTENT: String = fs::read_to_string("static/500.html")
        .unwrap_or(String::from(
        "SHIT'S FUCKED. BOTH AN INTERNAL ERROR AND UNABLE TO READ THE 505 PAGE. PAGE LUCIO, STAT."
    ));
}

#[derive(Debug, Template)]
#[template(path = "404.html")]
struct PageNotFound {
    user: Option<User>,
    original_uri: Uri,
}

fn page_not_found() -> (StatusCode, Html<String>) {
    (
        StatusCode::NOT_FOUND,
        PageNotFound {
            user: None,
            original_uri: Uri::from_static("/"),
        }
        .render()
        .unwrap_or(String::from(
            "404 PAGE CONTENT CRASHED ON COMPILATION. PAGE LUCIO, STAT.",
        ))
        .into(),
    )
}

fn request_timeout() -> impl IntoResponse {
    (
        StatusCode::REQUEST_TIMEOUT,
        "Request took too long".to_string(),
    )
}

fn bad_request(elaboration: String) -> impl IntoResponse {
    (
        StatusCode::BAD_REQUEST,
        format!("Bad request: {}", elaboration),
    )
}

fn unauthorized() -> impl IntoResponse {
    (StatusCode::UNAUTHORIZED,
    format!("Unauthorized! If you see this, you tried doing some action that needs to be logged in without being logged in. Log in dickhead. Also, if you see this, tell lucio to update this shit page."))
}

fn forbidden() -> impl IntoResponse {
    (
        StatusCode::FORBIDDEN,
        format!("You do not have the necessary permissions to do whatever you were trying to do."),
    )
}
