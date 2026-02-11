use std::collections::HashMap;

use crate::{user::UserType, utils::template_to_response, RootErrors, ServerState, User};
use askama::Template;
use axum::extract::{OriginalUri, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::NaiveDate;
use http::{StatusCode, Uri};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordProgressPin {
    link: String,
    date: NaiveDate,
}

#[derive(Deserialize)]
pub enum ProgressPin {
    OldNsfw,
    NewNsfw,
    OldSfw,
    NewSfw,
}

/// If an admin or uploader is logged in, shows the current archival progress
pub async fn view_archival_progress(
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

    // Admins and uploaders can both access this page.
    if !super::user_is_admin(&user)
        && user
            .as_ref()
            .is_none_or(|user| user.user_type != UserType::Uploader)
    {
        return Err(RootErrors::NotFound(original_uri, cookie_jar, user));
    }

    // Read all the pins from sql.
    const PROGRESS_PINS_SELECT_QUERY: &str =
        "SELECT * FROM arbitrary_value WHERE item_key LIKE 'art_archival_pin%';";
    let progress_pins = db_connection
        .query(PROGRESS_PINS_SELECT_QUERY, &[])
        .await
        .map_err(|err| {
            eprintln!("[ART ARCHIVAL PROJECT] Failed getting progress pins! {err:?}");
            RootErrors::InternalServerError
        })?;

    let map_of_pins: HashMap<String, DiscordProgressPin> = progress_pins
        .iter()
        .map(|row| {
            let name_of_pin: String = row.get("item_key");
            let pin: DiscordProgressPin = serde_json::from_str(row.get("item_value")).unwrap();

            (name_of_pin, pin)
        })
        .collect();

    Ok(template_to_response(ArtArchivalProgressPage {
        user,
        original_uri,

        old_nsfw_pin: map_of_pins.get("art_archival_pin_old_nsfw").unwrap(),
        new_nsfw_pin: map_of_pins.get("art_archival_pin_new_nsfw").unwrap(),
        old_sfw_pin: map_of_pins.get("art_archival_pin_old_sfw").unwrap(),
        new_sfw_pin: map_of_pins.get("art_archival_pin_new_sfw").unwrap(),
    }))
}

#[derive(Debug, Template)]
#[template(path = "admin/art_archival_project.html")]
struct ArtArchivalProgressPage<'a> {
    user: Option<User>,
    original_uri: Uri,

    old_nsfw_pin: &'a DiscordProgressPin,
    new_nsfw_pin: &'a DiscordProgressPin,
    old_sfw_pin: &'a DiscordProgressPin,
    new_sfw_pin: &'a DiscordProgressPin,
}

impl DiscordProgressPin {
    /// Given two progress pins, returns the distance between them in a human-readable format. May also include words of encouragement.
    /// Assumes that this date, and not `other``, is the smaller one.
    /// The output is something like "We have [X] days left to archive. Put otherwise, it's about [X] months, or [X] years."
    fn get_days_to_archive(&self, other: &Self) -> String {
        let distance = other.date - self.date;
        let days_of_difference = distance.num_days();

        if days_of_difference == 0 {
            return "<b>There is nothing else to archive here.</b> We archived it all. It's over. Thank you so much.".to_string();
        }

        if days_of_difference == 1 {
            return "We have <b>1 day</b> left to archive. You can do this! One final push!!"
                .to_string();
        }

        let mut human_readable_difference: String =
            format!("We have <b>{days_of_difference} days</b> left to archive. ");

        if days_of_difference < 0 {
            human_readable_difference.push_str(
                "Put otherwise, it's... <em>wait what?</em> Did someone put a wrong date on one of these?",
            );
        } else if days_of_difference < 30 {
            human_readable_difference.push_str("<b>We're at the finish line, you got this.</b> Archive a little today, and we'll soon be done.");
        } else {
            let months_of_difference = days_of_difference / 30;
            let s_if_multiple_months = if months_of_difference > 1 { "s" } else { "" };

            human_readable_difference.push_str(&format!(
                "Put otherwise, it's <b>about {months_of_difference} month{s_if_multiple_months}</b>"
            ));

            if months_of_difference == 1 {
                human_readable_difference.push('.');
            } else {
                let years_of_difference = months_of_difference / 12;
                let s_if_multiple_years = if years_of_difference > 1 { "s" } else { "" };

                human_readable_difference.push_str(&format!(
                    ", or <b>about {years_of_difference} year{s_if_multiple_years}</b>."
                ));
            }
        }

        human_readable_difference
    }

    /// Converts the web discord URL to one that opens in-app.
    fn get_clickable_discord_url(&self) -> String {
        self.link.replace("https://discord.com/", "discord://-/")
    }

    /// Converts the date to one that's easier to parse
    fn get_readable_date(&self) -> String {
        self.date.format("%B %d, %Y").to_string()
    }
}

impl ProgressPin {
    fn snake_case(&self) -> &str {
        match self {
            ProgressPin::OldNsfw => "old_nsfw",
            ProgressPin::NewNsfw => "new_nsfw",
            ProgressPin::OldSfw => "old_sfw",
            ProgressPin::NewSfw => "new_sfw",
        }
    }
}

#[derive(Deserialize)]
pub struct UpdatePinRequest {
    updated_pin: ProgressPin,

    #[serde(flatten)]
    pin_data: DiscordProgressPin,
}

/// Updates the current progress of archival.
#[axum::debug_handler]
pub async fn update_archival_progress(
    State(state): State<ServerState>,
    cookie_jar: tower_cookies::Cookies,
    Json(updated_pin): Json<UpdatePinRequest>,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_err| RootErrors::InternalServerError)?;

    let user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    if user.is_none() {
        return Err(RootErrors::Unauthorized);
    }

    // Admins and uploaders can both update this page.
    if !super::user_is_admin(&user)
        || user
            .as_ref()
            .is_some_and(|user| user.user_type == UserType::Uploader)
    {
        return Err(RootErrors::Forbidden);
    }

    // If it passed axum's JSON reader, I am going to assume it's valid info. If not, well, shit.
    const PROGRESS_PINS_UPDATE_QUERY: &str =
        "UPDATE arbitrary_value SET item_value=$1 WHERE item_key=$2;";

    let progress_pin_key = format!("art_archival_pin_{}", updated_pin.updated_pin.snake_case());
    let progress_pin_value = serde_json::to_string(&updated_pin.pin_data).map_err(|err| {
        eprintln!("[ART ARCHIVAL PROJECT] Failed converting progress pin value to string! Pin date: {:?}, err: {err:?}", updated_pin.pin_data);
        RootErrors::InternalServerError
    })?;

    let _ = db_connection
        .execute(
            PROGRESS_PINS_UPDATE_QUERY,
            &[&progress_pin_value, &progress_pin_key],
        )
        .await
        .map_err(|err| {
            eprintln!("[ART ARCHIVAL PROJECT] Failed getting progress pins! {err:?}");
            RootErrors::InternalServerError
        })?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
