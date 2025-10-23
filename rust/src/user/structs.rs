use chrono::{DateTime, Utc, Duration};
use std::net::SocketAddr;



pub struct User {
    pub username: String,
    pub profile_pic_s3_key: String, // The S3 key of their pfp image. Assumed to be in public bucket.
    pub discord_info: Option<UserOauth2>
}

pub struct UserSession {
    pub user: User,
    pub creation_time: DateTime<Utc>,
    pub session_ip: SocketAddr,
}

pub struct UserOauth2 {
    // TODO: Fill this in
    authorization_code: String
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
}
