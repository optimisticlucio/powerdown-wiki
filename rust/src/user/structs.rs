use chrono::{DateTime, Utc, Duration};
use std::net::{SocketAddr, SocketAddrV4};
use postgres::Row;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use postgres_types::{FromSql, ToSql, Type};
use std::env;
use tower_cookies::{Cookie, Cookies, cookie::SameSite};

use crate::{RootErrors, ServerState};

pub struct User {
    id: i64,
    /// The username is not unique and is just the display name. It can have spaces, etc! If you need something that's 100% tied to this user, use the ID.
    pub display_name: String,
    //pub profile_pic_s3_key: String, // The S3 key of their pfp image. Assumed to be in public bucket. // TODO
}

pub struct UserSession {
    pub user: User,
    pub creation_time: DateTime<Utc>,
    //pub session_ip: SocketAddr, // TODO: Implement
    /// A string representing this specific session. Long enough to be somewhat secure.
    pub session_id: String 

}

pub struct UserOauth2 {
    // TODO: Fill this in
    /// Part of the OAuth2 protocol; the token used to communicate with the resource owner.
    pub access_token: String,
    /// Part of the OAuth2 protocol; the token used to get new access tokens.
    pub refresh_token: String,
    pub provider: Oauth2Provider,

}

#[derive(FromSql, ToSql, Debug)]
pub enum Oauth2Provider {
    Discord,
    Google,
    Github
}

impl User {
    /// Given a user ID, returns the user, if it exists.
    pub async fn get_by_id(db_connection: &Object<Manager>, given_id: &str) -> Option<Self> {
        let query = "SELECT * FROM site_user WHERE id=$1";

        let resulted_row = db_connection.query_opt(query, &[&given_id])
                .await.unwrap(); // Can unwrap here because ID uniqueness enforced by DB.
        
        match resulted_row {
            None => None,
            Some(row) => Some(Self::from_row(row).await)
        }
    }

    /// Given a valid site_user row, converts it to a User struct.
    async fn from_row(row: Row) -> Self {
        Self {
            id: row.get("id"),
            display_name: row.get("display_name"),
        }
    }

    /// Given access to a request's cookie jar, attempts to get the logged in user.
    pub async fn get_from_cookie_jar(db_connection: &Object<Manager>, cookie_jar: &Cookies) -> Option<Self> {
        let user_session_id = cookie_jar.get("USER_SESSION_ID")?;
        let user_session = UserSession::get_by_id(&db_connection, user_session_id.value()).await?;

        Some(user_session.user)
    }

    /// Given access to a request's cookie jar, attempts to get the logged in user. Convenience function; get_from_cookie_jar version for the proper one.
    pub async fn easy_get_from_cookie_jar(state: &ServerState, cookie_jar: &Cookies) -> Result<Option<Self>, RootErrors> {
        let db_connection = state.db_pool.get().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;
        Ok(User::get_from_cookie_jar(&db_connection, &cookie_jar).await)
    }

    /// Creates a new user in the DB, returns the created user.
    pub async fn create_in_db(db_connection: &Object<Manager>, display_name: &str) -> Self {
        todo!("Didn't implement creating a user yet")
    }
}


impl UserSession {
    /// Checks whether the given user session is expired yet.
    fn is_expired(&self) -> bool {
        const SESSION_TIME_TO_EXPIRE: Duration = Duration::days(30);

        // Compare creation date + session length to the current UTC.
        self.creation_time.checked_add_signed(SESSION_TIME_TO_EXPIRE)
            .unwrap() // This'll only panic if we reach the year 3000. It'll be fine.
            <=  chrono::Utc::now()
    }

    /// Given a session ID, returns the session, if it exists.
    pub async fn get_by_id(db_connection: &Object<Manager>, given_id: &str) -> Option<Self> {
        let query = "SELECT * FROM user_session WHERE session_id=$1";

        let resulted_row = db_connection.query_opt(query, &[&given_id])
                .await.unwrap(); // Can unwrap here because ID uniqueness enforced by DB.
        
        match resulted_row {
            None => None,
            Some(row) => Some(Self::from_row(db_connection, row).await)
        }
    }

    /// Given a valid user_session row, converts it to a UserSession struct.
    async fn from_row(db_connection: &Object<Manager>, row: Row) -> Self {
        Self {
            // Existence of parent user is enforced by DB, unwrap allowed.
            user: User::get_by_id(db_connection, row.get("user_id")).await.unwrap(),
            session_id: row.get("session_id"),
            creation_time: row.get("creation_time"),
            //session_ip:  // TODO
        }
    }

    /// Starts a new user session for the given user, returns the session.
    pub async fn create_new_session(db_connection: &Object<Manager>, user: &User) -> Self {
        todo!("Didn't implement create_new_session")
    }

    /// Creates a cookie of the given session to pass to the user.
    pub fn to_cookie(&self) -> Cookie<'static> {
        let mut cookie = Cookie::new("USER_SESSION_ID", self.session_id.clone()); 
        
        cookie.set_same_site(SameSite::Strict);
        cookie.set_http_only(true);
        
        cookie
    }
}

impl UserOauth2 {
    /// Creates DB associations between the given OAuth2 data and the given user.
    pub async fn associate_with_user(&self, db_connection: &Object<Manager>, user: &User) -> () {
        todo!("Didn't implement associate_with_user")
    }
}

impl Oauth2Provider {
    /// Returns the relevant login URL for the given provider. References enviroment variables.
    pub fn get_user_login_url(&self) -> String {
        match self {
            Oauth2Provider::Discord => {
                let client_id = env::var("DISCORD_OAUTH2_CLIENT_ID").unwrap();
                let redirect_uri = format!("{}/user/oauth2/discord", env::var("WEBSITE_URL").unwrap());
                // The format for scopes is "scope1+scope2+scope3"
                const SCOPES: &'static str = "identify";

                let encoded_redirect_uri = urlencoding::encode(&redirect_uri);

                format!("https://discord.com/oauth2/authorize?client_id={client_id}&response_type=code&redirect_uri={encoded_redirect_uri}&scope={SCOPES}")
            }
            _ => {
                todo!("Requested an unimplemented oauth2 login");
            }
        }
    }

    /// Returns the relevant URL to send the access token. 
    pub fn get_token_url(&self) -> String {
        match self {
            Oauth2Provider::Discord => {
                "https://discord.com/api/oauth2/token".to_string()
            }
            _ => {
                todo!("Requested an unimplemented oauth2 token");
            }
        }
    }

    /// Given an access token, attempts to get a relevant user.
    pub async fn get_user_from_access_token(&self, db_connection: &Object<Manager>, access_token: &str) -> Option<User> {
        let query = "SELECT * FROM site_user INNER JOIN user_oauth ON site_user.id = user_oauth.user_id WHERE provider=$1 AND access_token=$2";

        let resulted_row = db_connection.query_opt(query, &[&self, &access_token])
                .await.unwrap(); // Can unwrap here because access token uniqueness enforced by DB.
        
        match resulted_row {
            None => None,
            Some(row) => Some(User::from_row(row).await)
        }
    }
}