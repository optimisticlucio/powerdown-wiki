use axum::{
    http::{response, StatusCode}, response::{Html, IntoResponse, Response}
};
use std::fs;
use lazy_static::lazy_static;
use askama::{Template};
use crate::user::User;

pub enum RootErrors {
    NOT_FOUND,
    INTERNAL_SERVER_ERROR,
    REQUEST_TIMEOUT
}

impl IntoResponse for RootErrors {
    fn into_response(self) -> Response {
        match self {
            Self::NOT_FOUND => page_not_found().into_response(),
            Self::INTERNAL_SERVER_ERROR => INTERNAL_SERVER_ERROR_PAGE_CONTENT.clone().into_response(),
            Self::REQUEST_TIMEOUT => request_timeout().into_response()
        }
    }
}


lazy_static! {
    static ref INTERNAL_SERVER_ERROR_PAGE_CONTENT: String = fs::read_to_string("static/500.html").unwrap_or(String::from("SHIT'S FUCKED. BOTH AN INTERNAL ERROR AND UNABLE TO READ THE 505 PAGE. PAGE LUCIO, STAT."));
}


#[derive(Template)] 
#[template(path = "404.html")]
struct PageNotFound {
    user: Option<User>
}

fn page_not_found() -> (StatusCode, Html<String>) {
    (
        StatusCode::NOT_FOUND, 
        PageNotFound {
            user: None
        }.render()
            .unwrap_or(String::from("404 PAGE CONTENT CRASHED ON COMPILATION. PAGE LUCIO, STAT.")).into()
    )
}

fn request_timeout() -> impl IntoResponse{
    (StatusCode::REQUEST_TIMEOUT,
    "Request took too long".to_string())
}