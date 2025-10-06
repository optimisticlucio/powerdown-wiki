use chrono::{DateTime, Datelike, Utc};
use axum::response::{Response, IntoResponse, Html};
use axum::body::Body;
use askama::Template;
use crate::errs::RootErrors;

pub fn format_date_to_human_readable(date: DateTime<Utc>) -> String {
    let readable_day = match date.day() {
        1 => "1st".to_owned(),
        2 => "2nd".to_owned(),
        3 => "3rd".to_owned(),
        x => format!("{x}th")
    };
    let readable_month = date.format("%B");

    format!("{readable_month} {readable_day}")
}

pub fn join_names_human_readable(names: Vec<&str>) -> String {
    match names.len() {
        0 => String::new(),
        1 => names[0].to_string(),
        2 => format!("{} and {}", names[0], names[1]),
        _ => format!("{}, and {}", names[..names.len()-1].join(", "), names.last().unwrap()),
    }
}

pub fn template_to_response<T: Template>(template: T) -> Response<Body> {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            eprintln!("Failed to render template: {err:?}");
            RootErrors::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}