use chrono::{DateTime, Utc, Duration};
use rand::{Rng};
use tower_cookies::cookie;
use std::net::{SocketAddr, SocketAddrV4};
use postgres::Row;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use postgres_types::{FromSql, ToSql, Type};
use std::env;
use tower_cookies::{Cookie, Cookies, cookie::SameSite};

use crate::{RootErrors, ServerState};

pub struct User {
    id: i32,
    pub user_type: UserType,

    /// The display name is not unique. It can have spaces, etc! If you need something that's 100% tied to this user, use the ID.
    pub display_name: String,
    //pub profile_pic_s3_key: String, // The S3 key of their pfp image. Assumed to be in public bucket. // TODO
}

#[derive(FromSql, ToSql, Debug)]
#[postgres(name="user_type", rename_all = "snake_case")]
pub enum UserType {
    Normal,
    Admin,
    SuperAdmin
}

pub struct UserSession {
    pub user: User,
    pub creation_time: DateTime<Utc>,
    //pub session_ip: SocketAddr, // TODO: Implement
    /// A string representing this specific session. Long enough to be somewhat secure.
    pub session_id: String 

}

static USER_SESSION_MAX_LENGTH: Duration = Duration::days(30);

pub struct UserOpenId {
    // TODO: Fill this in
    /// Part of the OpenID protocol; the sub is the user's ID, in function.
    pub sub: String,
    pub provider: Oauth2Provider,

}

#[derive(FromSql, ToSql, Debug)]
#[postgres(name="oauth_provider", rename_all = "snake_case")]
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
            Some(row) => Some(Self::from_row(row))
        }
    }

    /// Given a valid site_user row, converts it to a User struct.
    fn from_row(row: Row) -> Self {
        Self {
            id: row.get("id"),
            display_name: row.get("display_name"),
            user_type: row.get("user_type"),
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
        let query = "INSERT INTO site_user (id,display_name) VALUES ($1,$2) RETURNING *";

        // To make sure the ID is fully unique, we'll create it just before inserting and let the DB assure it is unique.
        // If the code verifies its uniqueness we run into all sorts of race conditions.
        loop {
            let random_user_id = rand::rng().random_range(1..i32::MAX); // Best not to have negative IDs.

            let result_of_insert = db_connection.query_one(query, &[&random_user_id, &display_name])
                .await;

            // If the insert was successful, return the created user!
            if let Ok(successful_user_row) = result_of_insert {
                return Self::from_row(successful_user_row);
            }
        }

    }
}


impl UserSession {
    /// Checks whether the given user session is expired yet.
    fn is_expired(&self) -> bool {
        // Compare creation date + session length to the current UTC.
        self.creation_time.checked_add_signed(USER_SESSION_MAX_LENGTH)
            .unwrap() // This'll only panic if we reach the year 3000. It'll be fine.
            <=  chrono::Utc::now()
    }

    /// Given a session ID, returns the session, if it exists.
    pub async fn get_by_id(db_connection: &Object<Manager>, given_id: &str) -> Option<Self> {
        const QUERY: &str = "SELECT * FROM user_session WHERE session_id=$1";

        let resulted_row = db_connection.query_opt(QUERY, &[&given_id])
                .await.unwrap(); // Can unwrap here because ID uniqueness enforced by DB.
        
        match resulted_row {
            None => None,
            Some(row) => {
                let parsed_session = Self::from_row(db_connection, row).await;

                // Validate the session is useable. If not, nuke it from the DB.
                if parsed_session.is_expired() {
                    const DELETE_QUERY: &str = "DELETE FROM user_session WHERE session_id=$1";

                    let _ = db_connection.execute(DELETE_QUERY, &[&given_id]).await;

                    None
                }   
                else {
                    Some(parsed_session)
                }
            }
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
        cookie.set_max_age(cookie::time::Duration::seconds_f64(USER_SESSION_MAX_LENGTH.as_seconds_f64()));
        
        cookie
    }
}

impl UserOpenId {
    /// Creates DB associations between the given OpenID data and the given user.
    pub async fn associate_with_user(&self, db_connection: &Object<Manager>, user: &User) -> () {
        const QUERY: &str = "INSERT INTO user_oauth (user_id,provider,sub) VALUES ($1,$2,$3)";

        let _ = db_connection.execute(QUERY, &[&user.id, &self.provider, &self.sub])
                .await.unwrap(); 
    }
}

impl Oauth2Provider {
    /// Returns the redirect URI we give the relevant provider.
    pub fn get_redirect_uri(&self) -> String {
        match self {
            Oauth2Provider::Discord => format!("{}/user/oauth2/discord", env::var("WEBSITE_URL").unwrap()),
            _ => {
                todo!("Requested an unimplemented oauth2 redirect URI");
            }
        }
    }

    /// Returns the relevant login URL for the given provider. References enviroment variables.
    pub fn get_user_login_url(&self) -> String {
        match self {
            Oauth2Provider::Discord => {
                let client_id = env::var("DISCORD_OAUTH2_CLIENT_ID").unwrap();
                let redirect_uri = self.get_redirect_uri();
                // The format for scopes is "scope1+scope2+scope3"
                const SCOPES: &'static str = "identify+openid";

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

    /// Given an OpenID sub, attempts to get a relevant user.
    pub async fn get_user_from_sub(&self, db_connection: &Object<Manager>, sub: &str) -> Option<User> {
        let query = "SELECT * FROM site_user INNER JOIN user_openid ON site_user.id = user_oauth.user_id WHERE provider=$1 AND sub=$2";

        let resulted_row = db_connection.query_opt(query, &[&self, &sub])
                .await.unwrap(); // Can unwrap here because access token uniqueness enforced by DB.
        
        resulted_row.map(User::from_row)
    }
}