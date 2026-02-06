use chrono::{DateTime, Duration, Utc};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use postgres::Row;
use postgres_types::{FromSql, ToSql};
use rand::distr::SampleString;
use rand::{distr::Alphanumeric, Rng};
use serde::Deserialize;
use std::env;
use tower_cookies::cookie;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

use crate::{RootErrors, ServerState};

/// Relative links to various default profile pictures users may have.
const USER_DEFAULT_PFPS: [&str; 9] = [
    "/static/img/pd_logo.svg",
    "/static/img/user/default_pfps/nikki.jpg",
    "/static/img/user/default_pfps/kate.jpg",
    "/static/img/user/default_pfps/fynn.jpg",
    "/static/img/user/default_pfps/casti.jpg",
    "/static/img/user/default_pfps/artemis.jpg",
    "/static/img/user/default_pfps/ucas.jpg",
    "/static/img/user/default_pfps/clyde.jpg",
    "/static/img/user/default_pfps/delphi.jpg",
];

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: i32,
    pub user_type: UserType,

    /// The display name is not unique. It can have spaces, etc! If you need something that's 100% tied to this user, use the ID.
    pub display_name: String,
    pub profile_pic_s3_key: Option<String>, // The S3 key of their pfp image. Assumed to be in public bucket.
    pub last_modified: DateTime<Utc>, // The last time that this user's info was modified.
    pub creator_name: Option<String>, // The name which identifies this user in art posts and such.
}

#[derive(FromSql, ToSql, Debug, Clone, Deserialize, PartialEq)]
#[postgres(name = "user_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]  // Add this line
pub enum UserType {
    /// The default user type someone gets when they first sign up.
    Normal, 
    /// A user who's been promoted by admins and allowed to post to the site.
    Uploader,
    /// Admins, tasked with handling daily tasks and making sure nothing blows up.
    Admin,
    /// Superadmins, people who can assign other admins and have root access.
    Superadmin,
}

/// A struct representing the permissions we give each user type. Should only be accessed using the .perms() function, and never constructed.
#[derive(Debug, Clone, Copy)]
pub struct UserPermissions {
    /// Whether the given user type can post art to the art section.
    pub can_post_art: bool,
    /// Whether the given user type can create new characters.
    pub can_post_characters: bool,
    /// Whether the given user type can modify the misc section of the site.
    pub can_modify_misc: bool,
    /// Whether the given user type can turn other people into admins.
    pub can_promote_to_admin: bool,
    /// Whether the given user type can modify info on other users, except turning into admin.
    pub can_modify_users: bool,
    /// Whether the given user can ban other users from the site.
    pub can_ban_users: bool,
    /// Whether the given user type can modify content posted by other users, like stories or art.
    pub can_modify_others_content: bool,
}

#[derive(Debug)]
pub struct UserSession {
    pub user: User,
    pub creation_time: DateTime<Utc>,
    //pub session_ip: SocketAddr, // TODO: Implement
    /// A string representing this specific session. Long enough to be somewhat secure.
    pub session_id: String,
}

static USER_SESSION_MAX_LENGTH: Duration = Duration::days(30);

/// A struct representing an association between a given user and an OAuth2 provider, will probably only use for login methods.
#[derive(Debug)]
pub struct OAuth2Association {
    /// The user ID, or any equivalent thereof, this given user has on the provider's database.
    pub provider_user_id: String,
    pub provider: Oauth2Provider,
}

#[derive(FromSql, ToSql, Debug, PartialEq)]
#[postgres(name = "oauth_provider", rename_all = "snake_case")]
pub enum Oauth2Provider {
    Discord,
    Google,
    Github,
}

impl User {
    /// Given a user ID, returns the user, if it exists.
    pub async fn get_by_id(db_connection: &Object<Manager>, given_id: &i32) -> Option<Self> {
        const GET_USER_QUERY: &str = "SELECT * FROM site_user WHERE id=$1";

        let resulted_row = db_connection.query_opt(GET_USER_QUERY, &[&given_id]).await.unwrap(); // Can unwrap here because ID uniqueness enforced by DB.

        match resulted_row {
            None => None,
            Some(row) => Some(Self::from_row(row)),
        }
    }

    /// Given a valid site_user row, converts it to a User struct.
    fn from_row(row: Row) -> Self {
        Self {
            id: row.get("id"),
            display_name: row.get("display_name"),
            user_type: row.get("user_type"),
            profile_pic_s3_key: row.get("profile_picture_s3_key"),
            creator_name: row.get("creator_name"),
            last_modified: row.get("last_modified_date"),
        }
    }

    /// Given access to a request's cookie jar, attempts to get the logged in user.
    pub async fn get_from_cookie_jar(
        db_connection: &Object<Manager>,
        cookie_jar: &Cookies,
    ) -> Option<Self> {
        let user_session_id = cookie_jar.get("USER_SESSION_ID")?;
        let user_session = UserSession::get_by_id(&db_connection, user_session_id.value()).await?;

        Some(user_session.user)
    }

    /// Given access to a request's cookie jar, attempts to get the logged in user. Convenience function; get_from_cookie_jar version for the proper one.
    pub async fn easy_get_from_cookie_jar(
        state: &ServerState,
        cookie_jar: &Cookies,
    ) -> Result<Option<Self>, RootErrors> {
        let db_connection = state
            .db_pool
            .get()
            .await
            .map_err(|_| RootErrors::InternalServerError)?;
        Ok(User::get_from_cookie_jar(&db_connection, &cookie_jar).await)
    }

    /// Creates a new user in the DB, returns the created user.
    pub async fn create_in_db(
        server_state: &ServerState,
        db_connection: &Object<Manager>,
        display_name: &str,
        pfp_file: Option<Vec<u8>>,
    ) -> Self {
        let query = "INSERT INTO site_user (id,display_name) VALUES ($1,$2) RETURNING *";

        // To make sure the ID is fully unique, we'll create it just before inserting and let the DB assure it is unique.
        // If the code verifies its uniqueness we run into all sorts of race conditions.
        let mut created_user: Option<User> = None;
        while created_user.is_none() {
            let random_user_id = rand::rng().random_range(1..i32::MAX); // Best not to have negative IDs.

            let result_of_insert = db_connection
                .query_one(query, &[&random_user_id, &display_name])
                .await;

            // If the insert was successful, return the created user!
            if let Ok(successful_user_row) = result_of_insert {
                created_user = Some(Self::from_row(successful_user_row));
            }
        }

        let successfully_created_user = created_user.unwrap();

        // User is created? Splendid. Now let's get some info that we're either unsure about or is dependent on the ID.

        // TODO - HANDLE PFP

        successfully_created_user
    }

    /// Returns a URL pointing towards a default pfp image.
    pub fn get_default_pfp_url(&self) -> &'static str {
        let default_pfp_index = (self.id.abs() as usize) % USER_DEFAULT_PFPS.len();

        return USER_DEFAULT_PFPS[default_pfp_index];
    }

    /// Returns a URL pointing towards the given user's PFP.
    pub fn get_pfp_url(&self) -> String {
        if let Some(profile_pic_s3_key) = &self.profile_pic_s3_key {
            crate::utils::get_s3_public_object_url(profile_pic_s3_key)
        } else {
            self.get_default_pfp_url().to_string()
        }
    }

    /// Returns a URL pointing towards this user's user page.
    pub fn get_user_page_url(&self) -> String {
        format!("{}/user/{}",
            std::env::var("WEBSITE_URL").unwrap(),
            self.id
            )
    }

    // Returns whether another user can modify this user's type.
    pub fn can_have_user_type_modified_by(&self, other: &Self) -> bool {
        // Can't change your own permissions.
        if self == other {
            return false;
        }

        match self.user_type {
            // Can't demote superadmins without DB access
            UserType::Superadmin => false,
            // Only those who can promote someone else to admin can modify the perms of other admins.
            UserType::Admin => other.user_type.permissions().can_promote_to_admin,

            _ => other.user_type.permissions().can_modify_users
        }
    }

    /// Returns whether a given user can modify this user's visible data. (Pfp, nickname, etc)
    pub fn can_have_visible_data_modified_by(&self, other: &Self) -> bool {
        self == other || other.user_type.permissions().can_modify_users
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl UserType {
    /// Returns the given user type's permissions.
    pub fn permissions(&self) -> UserPermissions {
        match self {
            Self::Normal => UserPermissions {
                can_post_art: false,
                can_post_characters: false,
                can_modify_misc: false,
                can_ban_users: false,
                can_promote_to_admin: false,
                can_modify_users: false,
                can_modify_others_content: false,
            },
            Self::Uploader => UserPermissions {
                can_post_art: true,
                can_post_characters: true,
                can_modify_misc: false,
                can_ban_users: false,
                can_promote_to_admin: false,
                can_modify_users: false,
                can_modify_others_content: false,
            },
            Self::Admin => UserPermissions {
                can_post_art: true,
                can_post_characters: true,
                can_modify_misc: true,
                can_ban_users: true,
                can_modify_users: true,
                can_promote_to_admin: false,
                can_modify_others_content: true,
            },
            Self::Superadmin => UserPermissions {
                can_post_art: true,
                can_post_characters: true,
                can_modify_misc: true,
                can_ban_users: true,
                can_modify_users: true,
                can_promote_to_admin: true,
                can_modify_others_content: true,
            },
        }
    }
}

impl std::fmt::Display for UserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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

        let resulted_row = db_connection.query_opt(QUERY, &[&given_id]).await.unwrap(); // Can unwrap here because ID uniqueness enforced by DB.

        match resulted_row {
            None => None,
            Some(row) => {
                let parsed_session = Self::from_row(db_connection, row).await;

                // Validate the session is useable. If not, nuke it from the DB.
                if parsed_session.is_expired() {
                    const DELETE_QUERY: &str = "DELETE FROM user_session WHERE session_id=$1";

                    let _ = db_connection.execute(DELETE_QUERY, &[&given_id]).await;

                    None
                } else {
                    Some(parsed_session)
                }
            }
        }
    }

    /// Given a valid user_session row, converts it to a UserSession struct.
    async fn from_row(db_connection: &Object<Manager>, row: Row) -> Self {
        Self {
            // Existence of parent user is enforced by DB, unwrap allowed.
            user: User::get_by_id(db_connection, &row.get("user_id"))
                .await
                .unwrap(),
            session_id: row.get("session_id"),
            creation_time: row.get("creation_time"),
            //session_ip:  // TODO
        }
    }

    /// Starts a new user session for the given user, returns the session.
    pub async fn create_new_session(db_connection: &Object<Manager>, user: &User) -> Self {
        const QUERY: &str =
            "INSERT INTO user_session (user_id, session_id) VALUES ($1, $2) RETURNING *";

        loop {
            let random_session_id = Alphanumeric.sample_string(&mut rand::rng(), 64);

            let resulted_row = db_connection
                .query_one(QUERY, &[&user.id, &random_session_id])
                .await;

            if let Ok(successful_insert) = resulted_row {
                return Self::from_row(db_connection, successful_insert).await;
            }
        }
    }

    /// Creates a cookie of the given session to pass to the user.
    pub fn to_cookie(&self) -> Cookie<'static> {
        let mut cookie = Cookie::new("USER_SESSION_ID", self.session_id.clone());

        cookie.set_same_site(SameSite::Lax);
        cookie.set_http_only(true);
        cookie.set_max_age(cookie::time::Duration::seconds_f64(
            USER_SESSION_MAX_LENGTH.as_seconds_f64(),
        ));
        cookie.set_path("/");

        cookie
    }
}

impl OAuth2Association {
    /// Creates DB associations between the given OpenID data and the given user.
    pub async fn associate_with_user(&self, db_connection: &Object<Manager>, user: &User) -> () {
        const QUERY: &str =
            "INSERT INTO user_oauth_association (user_id,provider,oauth_user_id) VALUES ($1,$2,$3)";

        let _ = db_connection
            .execute(QUERY, &[&user.id, &self.provider, &self.provider_user_id])
            .await
            .unwrap();
    }
}

impl Oauth2Provider {
    /// Returns the redirect URI we give the relevant provider.
    pub fn get_redirect_uri(&self) -> String {
        match self {
            Oauth2Provider::Discord => {
                format!("{}/user/oauth2/discord", env::var("WEBSITE_URL").unwrap())
            }
            Oauth2Provider::Google => {
                format!("{}/user/oauth2/google", env::var("WEBSITE_URL").unwrap())
            }
            Oauth2Provider::Github => {
                format!("{}/user/oauth2/github", env::var("WEBSITE_URL").unwrap())
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
                const SCOPES: &'static str = "identify";

                let encoded_redirect_uri = urlencoding::encode(&redirect_uri);

                format!("https://discord.com/oauth2/authorize?client_id={client_id}&response_type=code&redirect_uri={encoded_redirect_uri}&scope={SCOPES}")
            }
            Oauth2Provider::Google => {
                let client_id = env::var("GOOGLE_OAUTH2_CLIENT_ID").unwrap();
                let redirect_uri = self.get_redirect_uri();
                // The format for scopes is "scope1+scope2+scope3"
                const SCOPES: &'static str = "https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile openid";

                let encoded_redirect_uri = urlencoding::encode(&redirect_uri);
                let encoded_scopes = urlencoding::encode(&SCOPES);

                format!("https://accounts.google.com/o/oauth2/auth?client_id={client_id}&response_type=code&redirect_uri={encoded_redirect_uri}&scope={encoded_scopes}")
            }
            Oauth2Provider::Github => {
                let client_id = env::var("GITHUB_OAUTH2_CLIENT_ID").unwrap();

                let redirect_uri = self.get_redirect_uri();
                let encoded_redirect_uri = urlencoding::encode(&redirect_uri);

                format!("https://github.com/login/oauth/authorize?client_id={client_id}&redirect_uri={encoded_redirect_uri}")
            }
        }
    }

    /// Returns the relevant URL to send the access token.
    pub fn get_token_url(&self) -> String {
        match self {
            Oauth2Provider::Discord => "https://discord.com/api/oauth2/token".to_string(),
            Oauth2Provider::Google => "https://oauth2.googleapis.com/token".to_string(),
            Oauth2Provider::Github => "https://github.com/login/oauth/access_token".to_string(),
        }
    }

    /// Returns the relevant URL to send the "who is this person" request after we got the access token
    pub fn get_identification_url(&self) -> String {
        match self {
            Oauth2Provider::Discord => "https://discord.com/api/users/@me".to_string(),
            Oauth2Provider::Google => "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            Oauth2Provider::Github => "https://api.github.com/user".to_string(),
        }
    }

    /// Returns a URL pointing towards an existing pfp the user has on the provider, if one exists
    pub async fn get_existing_user_pfp(&self, access_token: &str) -> Option<String> {
        match self {
            _ => None,
        }
    }

    /// Given a user ID, attempts to get a relevant user.
    pub async fn get_user_by_association(
        &self,
        db_connection: &Object<Manager>,
        oauth_user_id: &str,
    ) -> Option<User> {
        let query = "SELECT * FROM site_user INNER JOIN user_oauth_association ON site_user.id = user_oauth_association.user_id WHERE provider=$1 AND oauth_user_id=$2";

        let resulted_row = db_connection
            .query_opt(query, &[&self, &oauth_user_id])
            .await
            .unwrap(); // Can unwrap here because access token uniqueness enforced by DB.

        resulted_row.map(User::from_row)
    }
}
