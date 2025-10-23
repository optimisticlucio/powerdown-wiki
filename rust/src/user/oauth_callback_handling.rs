use crate::RootErrors;
use axum::response::IntoResponse;

/// Recieves the Discord Oauth callback. If user exists, adds a new login method for them. If not, creates a new user with their basic info.
pub async fn discord() -> Result<impl IntoResponse, RootErrors> {
    Ok("TODO: Implement discord oauth handling") // TODO
}