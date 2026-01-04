use std::env;
use crate::{RootErrors, ServerState, errs, user::{User, structs::{Oauth2Provider, OAuth2Association, UserSession}}};
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
#[axum::debug_handler]
pub async fn discord(
    State(state): State<ServerState>, 
    Query(query): Query<DiscordOauthQuery>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    // Did the user give us an authorization code?
    let authorization_code = query.code.ok_or_else(|| RootErrors::BAD_REQUEST(original_uri, Cookies::default(), "Entered Discord Authorization Callback without an authorization code.".to_string()))?;

    // First, talk to the Discord servers to see what account we just got access to.
    let discord_access_token_request_client = reqwest::ClientBuilder::default()
        .build().map_err(|err| {
            println!("[OAUTH2; DISCORD] Failed to build request client: {:?}", err);
            RootErrors::INTERNAL_SERVER_ERROR
        })?;

    let discord_response = discord_access_token_request_client
        .post(Oauth2Provider::Discord.get_token_url())
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &authorization_code),
            ("redirect_uri", &Oauth2Provider::Discord.get_redirect_uri())
        ])
        .basic_auth(env::var("DISCORD_OAUTH2_CLIENT_ID").unwrap(), env::var("DISCORD_OAUTH2_CLIENT_SECRET").ok())
        .send().await
        .map_err(|err| {
            println!("[OAUTH2; DISCORD] Failed sending request for access token: {:?}", err.to_string());
            RootErrors::INTERNAL_SERVER_ERROR
        })?;
    
    let text_response = discord_response.text().await.unwrap();
    // For some reason, converting the response to json directly results in a parse error. Can't wrap my head around it, but this seems to work.
    let discord_tokens: OAuthTokens =  serde_json::from_str(&text_response)
        .map_err(|err| {
            println!("[OAUTH2; DISCORD] Failed reading access token response: {:?}", err.to_string());
            RootErrors::INTERNAL_SERVER_ERROR
        })?;

    // Now we send another message to discord: "Who tf is this person?"

    let discord_identify_request = reqwest::ClientBuilder::default()
        .build().map_err(|err| {
            println!("[OAUTH2; DISCORD] Failed to build identification request client: {:?}", err);
            RootErrors::INTERNAL_SERVER_ERROR
        })?
        .get("https://discord.com/api/users/@me") // The API to get user info
        .header("Authorization", format!("Bearer {}", &discord_tokens.access_token))
        .send().await
        .map_err(|err| {
            println!("[OAUTH2; DISCORD] Failed sending identification request: {:?}", err.to_string());
            RootErrors::INTERNAL_SERVER_ERROR
        })?;

    let discord_user: DiscordUser = discord_identify_request
        .json().await
        .map_err(|err| {
            println!("[OAUTH2; DISCORD] Failed reading user's @me info: {:?}", err.to_string());
            RootErrors::INTERNAL_SERVER_ERROR
        })?;

    let db_connection = state.db_pool.get().await.unwrap();

    // Did this discord user create an account already?
    let access_token_user: Option<User> = Oauth2Provider::Discord.get_user_by_association(
        &db_connection,
        &discord_user.id).await; 

    if let Some(existing_user_with_connection) = access_token_user {
        // This connection exists in the DB.
        todo!("Implement case where db connection exists.")
    }
    else {
        // This connection does not exist in the DB.
        // Firstly, find or create the account to connect this to.
        let account_in_db = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

        let account_to_connect_to = if account_in_db.is_some() { account_in_db.unwrap() } else {
            let display_name = discord_user.global_name.unwrap_or(discord_user.username);
            User::create_in_db(&db_connection, &display_name).await
        };

        // Connect the OAuth method to the relevant user.
        OAuth2Association {
            provider: Oauth2Provider::Discord,
            provider_user_id: /*discord_tokens.sub*/ "TODO".to_string(),
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
#[serde(deny_unknown_fields)]
pub struct OAuthTokens {
    access_token: String,
    token_type: String,
    #[serde(default)]
    expires_in: Option<u64>,
    refresh_token: String,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
}

#[derive(Deserialize)] 
/// The info we get from discord after running users/@me, and more specifically, the info we care for
pub struct DiscordUser {
    id: String,
    username: String,
    global_name: Option<String>,
}
