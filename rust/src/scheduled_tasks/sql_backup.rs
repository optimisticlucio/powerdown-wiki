use crate::ServerState;
use super::get_current_human_readable_time;

/// When run, backs up the entire sql db to the private S3 bucket.
pub async fn backup_db(state: ServerState) {
    println!("[SQL BACKUP] System time is {}, initiating sql backup.", get_current_human_readable_time());
    // TODO: Implement

    /* TODO: Also, clean up any superflous backups (1 a day for this week. 1 a week for this month. 1 per month afterwards.)
    Likely best to do this action as a different function, but writing it here so I don't forget.*/
}