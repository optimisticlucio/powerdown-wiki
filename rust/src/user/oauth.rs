use std::env;
use crate::{RootErrors, ServerState, errs, user::{User, structs::{Oauth2Provider, UserOpenId, UserSession}}};
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
    cookie_jar: Cookies,
) -> Result<Response, RootErrors> {
    // Did the user give us an authorization code?
    let authorization_code = query.code.ok_or( RootErrors::BAD_REQUEST("Entered Discord Authorization Callback without an authorization code.".to_string()))?;

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
        
    let discord_tokens: OAuthTokens =  discord_response.json().await
        .map_err(|err| {
            println!("[OAUTH2; DISCORD] Failed reading access token response: {:?}", err.to_string());
            RootErrors::INTERNAL_SERVER_ERROR
        })?;

    // Now, parse the openID token to something readable.

    todo!("i hath headache");
    /*let discord_decode_key = jsonwebtoken::DecodingKey::

    let discord_openid: OpenIDTokenClaims = jsonwebtoken::decode(
        &discord_tokens.id_token, 
        key, 
        validation)
        .unwrap(); // TODO: Convert to map err*/

    let db_connection = state.db_pool.get().await.unwrap();


    // Did this discord user create an account already?
    let access_token_user = Oauth2Provider::Discord.get_user_from_sub(
        &db_connection,
        &discord_tokens.sub).await;

    if let Some(existing_user_with_connection) = access_token_user {
        // This connection exists in the DB.
        todo!("Implement case where db connection exists.")
    }
    else {
        // This connection does not exist in the DB.
        // Firstly, find or create the account to connect this to.
        let account_in_db = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

        let account_to_connect_to = if account_in_db.is_some() { account_in_db.unwrap() } else {
            let display_name = "DIDNT_IMPLEMENT_GETTING_DISPLAYNAME_YET";
            User::create_in_db(&db_connection, &display_name).await
        };

        // Connect the OAuth method to the relevant user.
        UserOpenId {
            provider: Oauth2Provider::Discord,
            sub: discord_tokens.sub,
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
    id_token: String, // the OpenID token
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenIDTokenClaims {
    sub: String,           // Subject (user ID)
    iss: String,           // Issuer
    aud: String,           // Audience
    exp: usize,            // Expiration time
    iat: usize,            // Issued at
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
}