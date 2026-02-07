use postgres_types::{FromSql, ToSql};
use serde::{Deserialize};


/// Enum representing the state of various user posts, like art, characters, and stories.
#[derive(Clone, FromSql, ToSql, Deserialize, Debug)]
#[postgres(name = "post_state", rename_all = "snake_case")]
pub enum PostState {
    Public,          // Publicly viewable, standard state.
    PendingApproval, // User-uploaded, pending admin review to be moved to public. Not visible.
    Processing,      // Currently mid-process by the server and/or database. Should not be viewable.
}

impl Default for PostState {
    fn default() -> Self {
        Self::Public
    }
}