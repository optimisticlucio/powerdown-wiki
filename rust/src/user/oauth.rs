use crate::{RootErrors, ServerState, user::{User, structs::{Oauth2Provider, UserOauth2, UserSession}}};
use axum::{Router, extract::{OriginalUri, Query, State}, response::{IntoResponse, Redirect, Response}, routing::get};
use tower_cookies::{Cookies};
use axum_extra::routing::RouterExt;
use serde::{Deserialize, Serialize};

pub fn router() -> Router<ServerState> {
    Router::new().route_with_tsr("/discord", get(discord))
}

/// Recieves the Discord Oauth callback. 
/// If user isn't logged in, and an account with these values exist, logs in. If an account with these values doesn't exist, creates one.
/// If the user is logged in, and an account with these values doesn't exist, connects this oauth to the logged in account.
/// If the user is logged in and this oauth method already exists for someone else, throws an error.
pub async fn discord(
    State(state): State<ServerState>, 
    Query(query): Query<DiscordOauthQuery>,
    cookie_jar: Cookies,
) -> Result<Response, RootErrors> {
    // Did the user give us an authorization code?
    let authorization_code = query.code.ok_or( RootErrors::BAD_REQUEST("Entered Discord Authorization Callback without an authorization code.".to_string()))?;

    // First, talk to the Discord servers to see what account we just got access to.
    let discord_access_token_request_client = reqwest::Client::builder()
        .build().map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

    let discord_tokens: OAuthTokens = discord_access_token_request_client
        .post(Oauth2Provider::Discord.get_token_url())
        // TODO: Insert relevant info
        .send().await
        .map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?
        .json().await
        .map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

    let db_connection = state.db_pool.get().await.unwrap();

    // Did this discord user create an account already?
    let access_token_user = Oauth2Provider::Discord.get_user_from_access_token(
        &db_connection,
        &discord_tokens.access_token).await;

    if let Some(existing_user_with_connection) = access_token_user {
        // This connection exists in the DB.
        todo!("Implement case where db connection exists.")
    }
    else {
        // This connection does not exist in the DB.
        // Firstly, find or create the account to connect this to.
        let account_in_db = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

        let account_to_connect_to = if account_in_db.is_some() { account_in_db.unwrap() } else {
            let display_name = "TODO: Properly get display name";
            User::create_in_db(&db_connection, &display_name).await
        };

        // Connect the OAuth method to the relevant user.
        UserOauth2 {
            provider: Oauth2Provider::Discord,
            access_token: discord_tokens.access_token,
            refresh_token: discord_tokens.refresh_token,
        }.associate_with_user(&db_connection, &account_to_connect_to).await;

        let user_session = UserSession::create_new_session(&db_connection, &account_to_connect_to).await;

        cookie_jar.add(user_session.to_cookie());

        Ok(Redirect::to("/user").into_response())
    }
}

/// Struct to handle Discord's query response to the oauth2 login.
#[derive(Deserialize)]
pub struct DiscordOauthQuery {
    state: Option<String>,
    /// The authorization code we send to discord to get the access token and refresh token.
    code: Option<String>, 
}

/// Struct to handle the end of the oauth handshake
#[derive(Deserialize)]
pub struct OAuthTokens {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
    refresh_token: String,
}