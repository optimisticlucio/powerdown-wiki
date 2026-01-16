use crate::{
    user::{
        structs::{OAuth2Association, Oauth2Provider, UserSession},
        User,
    },
    RootErrors, ServerState,
};
use axum::{
    extract::{OriginalUri, Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use axum_extra::routing::RouterExt;
use http::{header::USER_AGENT, Uri};
use serde::Deserialize;
use std::env;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route_with_tsr("/discord", get(discord))
        .route_with_tsr("/google", get(google))
        .route_with_tsr("/github", get(github))
}

/// Recieves the Discord Oauth callback.
/// If user isn't logged in, and an account with these values exist, logs in. If an account with these values doesn't exist, creates one.
/// If the user is logged in, and an account with these values doesn't exist, connects this oauth to the logged in account.
/// If the user is logged in and this oauth method already exists, throws an error.
#[axum::debug_handler]
pub async fn discord(
    State(state): State<ServerState>,
    Query(query): Query<OAuthQuery>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    oauth_process(
        "Discord",
        |discord_user: &DiscordUser| {
            discord_user
                .global_name
                .as_ref()
                .unwrap_or(&discord_user.username)
                .clone()
        },
        |discord_user: &DiscordUser| discord_user.id.clone(),
        Oauth2Provider::Discord,
        "DISCORD_OAUTH2_CLIENT_ID",
        "DISCORD_OAUTH2_CLIENT_SECRET",
        state,
        query,
        original_uri,
        cookie_jar,
    )
    .await
}

#[derive(Debug, Deserialize)]
/// The info we get from discord after running users/@me, and more specifically, the info we care for
pub struct DiscordUser {
    id: String,
    username: String,
    global_name: Option<String>,
}

/// Recieves the Google Oauth callback.
/// If user isn't logged in, and an account with these values exist, logs in. If an account with these values doesn't exist, creates one.
/// If the user is logged in, and an account with these values doesn't exist, connects this oauth to the logged in account.
/// If the user is logged in and this oauth method already exists, throws an error.
#[axum::debug_handler]
pub async fn google(
    State(state): State<ServerState>,
    Query(query): Query<OAuthQuery>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    oauth_process(
        "Google",
        |user: &GoogleUser| user.given_name.clone(),
        |user: &GoogleUser| user.id.clone(),
        Oauth2Provider::Google,
        "GOOGLE_OAUTH2_CLIENT_ID",
        "GOOGLE_OAUTH2_CLIENT_SECRET",
        state,
        query,
        original_uri,
        cookie_jar,
    )
    .await
}

#[derive(Debug, Deserialize)]
pub struct GoogleUser {
    id: String,
    email: String,
    name: String,       // Their actual IRL full name
    given_name: String, // First name
    picture: String,    // URL to their pfp image
}

/// Recieves the Github Oauth callback.
/// If user isn't logged in, and an account with these values exist, logs in. If an account with these values doesn't exist, creates one.
/// If the user is logged in, and an account with these values doesn't exist, connects this oauth to the logged in account.
/// If the user is logged in and this oauth method already exists, throws an error.
#[axum::debug_handler]
pub async fn github(
    State(state): State<ServerState>,
    Query(query): Query<OAuthQuery>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    oauth_process(
        "Github",
        |user: &GithubUser| user.login.clone(),
        |user: &GithubUser| user.id.to_string(),
        Oauth2Provider::Github,
        "GITHUB_OAUTH2_CLIENT_ID",
        "GITHUB_OAUTH2_CLIENT_SECRET",
        state,
        query,
        original_uri,
        cookie_jar,
    )
    .await
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    login: String, // The username
    id: i32,
    avatar_url: String,
}

async fn oauth_process<
    'a,
    T: serde::de::DeserializeOwned,
    U: FnOnce(&T) -> String,
    F: FnOnce(&T) -> String,
>(
    process_name_for_debug: &'a str,
    get_display_name: U,
    get_user_id: F,
    provider: Oauth2Provider,
    client_id_cookie_name: &'a str,
    client_secret_cookie_name: &'a str,
    state: ServerState,
    query: OAuthQuery,
    original_uri: Uri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();
    let logged_in_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    // Did the user give us an authorization code?
    let authorization_code = match query.code {
        None => {
            return Err(RootErrors::BadRequest(
                original_uri,
                cookie_jar,
                logged_in_user,
                format!(
                    "Entered {} Authorization Callback without an authorization code.",
                    process_name_for_debug
                ),
            ))
        }
        Some(x) => x,
    };

    // First, talk to the Google servers to see what account we just got access to.
    let access_token_request_client = reqwest::ClientBuilder::default().build().map_err(|err| {
        println!(
            "[OAUTH2; {}] Failed to build request client: {:?}",
            process_name_for_debug, err
        );
        RootErrors::InternalServerError
    })?;

    let response = access_token_request_client
        .post(provider.get_token_url())
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &authorization_code),
            ("redirect_uri", &provider.get_redirect_uri()),
        ])
        .basic_auth(
            env::var(client_id_cookie_name).unwrap(),
            env::var(client_secret_cookie_name).ok(),
        )
        .send()
        .await
        .map_err(|err| {
            println!(
                "[OAUTH2; {}] Failed sending request for access token: {:?}",
                process_name_for_debug,
                err.to_string()
            );
            RootErrors::InternalServerError
        })?;

    let text_response = response.text().await.unwrap();
    // For some reason, converting the response to json directly results in a parse error. Can't wrap my head around it, but this seems to work.
    let tokens: OAuthTokens = serde_json::from_str(&text_response)
        .or_else(|_| {
            // It's not JSON. Is it a URL-encoded form data?
            serde_urlencoded::from_str(&text_response)
        })
        .map_err(|err| {
            println!(
                "[OAUTH2; {}] Failed reading access token response: {:?}",
                process_name_for_debug,
                err.to_string()
            );
            RootErrors::InternalServerError
        })?;

    // Now we send another message: "Who tf is this person?"
    let identify_request = reqwest::ClientBuilder::default()
        .build()
        .map_err(|err| {
            println!(
                "[OAUTH2; {}] Failed to build identification request client: {:?}",
                process_name_for_debug, err
            );
            RootErrors::InternalServerError
        })?
        .get(provider.get_identification_url())
        .header("Authorization", format!("Bearer {}", &tokens.access_token))
        .header(USER_AGENT, "powerdown-wiki")
        .send()
        .await
        .map_err(|err| {
            println!(
                "[OAUTH2; {}] Failed sending identification request: {:?}",
                process_name_for_debug,
                err.to_string()
            );
            RootErrors::InternalServerError
        })?;

    let user_info: T = identify_request.json().await.map_err(|err| {
        println!(
            "[OAUTH2; {}] Failed reading user's @me info: {:?}",
            process_name_for_debug,
            err.to_string()
        );
        RootErrors::InternalServerError
    })?;

    let user_id = get_user_id(&user_info);

    // Did this user create an account already?
    let access_token_user: Option<User> = provider
        .get_user_by_association(&db_connection, &user_id)
        .await;

    if let Some(existing_user_with_connection) = access_token_user {
        // This connection exists in the DB.

        // If the user is logged in, some error is gonna be thrown.
        if let Some(logged_in_user) = logged_in_user {
            if logged_in_user == existing_user_with_connection {
                Err(RootErrors::BadRequest(
                    original_uri,
                    cookie_jar,
                    Some(logged_in_user),
                    "You're already logged in, silly! You can't re-log-in!".to_string(),
                ))
            } else {
                Err(RootErrors::BadRequest(
                    original_uri,
                    cookie_jar,
                    Some(logged_in_user),
                    "Someone already has an account with that discord account attached to it! Are you sure you didn't make two accounts by accident?".to_string()))
            }
        }
        // If the user isn't logged in, log in as usual.
        else {
            let user_session =
                UserSession::create_new_session(&db_connection, &existing_user_with_connection)
                    .await;

            cookie_jar.add(user_session.to_cookie());

            Ok(Redirect::to("/user").into_response())
        }
    } else {
        // This connection does not exist in the DB.
        // TODO: Check if logged in user has a different connection already. If so, throw an error.

        // If the user isn't logged in, create a new account for them.
        let account_to_connect_to = if logged_in_user.is_some() {
            logged_in_user.unwrap()
        } else {
            // TODO: Download pfp. Insert into the following variable:
            let user_existing_pfp: Option<Vec<u8>> = None;

            User::create_in_db(
                &state,
                &db_connection,
                &get_display_name(&user_info),
                user_existing_pfp,
            )
            .await
        };

        // -- THE LUCIO OVERRIDE --
        // For debugging where I'll need to recreate the DB often:
        // If Lucio (me) logs into an account, make it superadmin instantly.
        // Remove this once we go into production and I can set myself as superadmin in the DB.

        if provider == Oauth2Provider::Discord && user_id == "1352633375243899006" {
            const SUPERADMIN_QUERY: &str = "UPDATE site_user SET user_type='superadmin' WHERE id=$1";
            let _ = db_connection.execute(SUPERADMIN_QUERY, &[&account_to_connect_to.id]).await
                .map_err(|err| {
                    eprintln!("[LUCIO OVERRIDE] Failed to make user superadmin! {:?}. Continuing as normal.", err);
                });
        }

        // ------------------------

        // Connect the OAuth method to the user we now have.
        OAuth2Association {
            provider: provider,
            provider_user_id: user_id,
        }
        .associate_with_user(&db_connection, &account_to_connect_to)
        .await;

        let user_session =
            UserSession::create_new_session(&db_connection, &account_to_connect_to).await;

        cookie_jar.add(user_session.to_cookie());

        Ok(Redirect::to("/user").into_response())
    }
}
/// Struct to handle query response to the oauth2 login.
#[derive(Debug, Deserialize)]
pub struct OAuthQuery {
    state: Option<String>,
    /// The authorization code we send to discord to get the access token and refresh token.
    code: Option<String>,
}

/// Struct to handle the end of the oauth handshake
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OAuthTokens {
    access_token: String,
    token_type: String,
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
}
