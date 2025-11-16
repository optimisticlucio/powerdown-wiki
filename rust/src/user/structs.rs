use chrono::{DateTime, Utc, Duration};
use std::net::{SocketAddr, SocketAddrV4};
use postgres::Row;
use deadpool::managed::Object;
use deadpool_postgres::Manager;

pub struct User {
    id: i64,
    pub username: String,
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
    authorization_code: String
}

impl User {
    /// Given a user ID, returns the user, if it exists.
    pub async fn get_by_id(db_connection: Object<Manager>, given_id: &str) -> Option<Self> {
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
            username: row.get("username"),
        }
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
    pub async fn get_by_id(db_connection: Object<Manager>, given_id: &str) -> Option<Self> {
        let query = "SELECT * FROM user_session WHERE session_id=$1";

        let resulted_row = db_connection.query_opt(query, &[&given_id])
                .await.unwrap(); // Can unwrap here because ID uniqueness enforced by DB.
        
        match resulted_row {
            None => None,
            Some(row) => Some(Self::from_row(db_connection, row).await)
        }
    }

    /// Given a valid user_session row, converts it to a UserSession struct.
    async fn from_row(db_connection: Object<Manager>, row: Row) -> Self {
        Self {
            // Existence of parent user is enforced by DB, unwrap allowed.
            user: User::get_by_id(db_connection, row.get("user_id")).await.unwrap(),
            session_id: row.get("session_id"),
            creation_time: row.get("creation_time"),
            //session_ip:  // TODO
        }
    }
}
