use std::env;

use tokio::process::Command;

use crate::ServerState;
use super::get_current_human_readable_time;

/// Runs all processes related to SQL backup, including cleaning up old revisions.
pub async fn run_backup_processes(state: ServerState) {
    if let Err(err) = backup_db(state.clone()).await {
        eprintln!("[SQL BACKUP PROCESSES] Failed to backup DB! err: {:?}",err)
    }
    // TODO: Also, clean up any superflous backups from S3 (1 a day for this week. 1 a week for this month. 1 per month afterwards.)
}


/// When run, backs up the entire sql db to the private S3 bucket.
async fn backup_db(state: ServerState) -> Result<(), std::io::Error> {
    println!("[SQL BACKUP] System time is {}, initiating sql backup.", get_current_human_readable_time());

    const PG_DUMP_FILENAME: &str = "pg_dump";

    // The command currently assumes the DB is a parallel docker container.
    // If we ever move off of docker, make this command read the ENV for the DB and use it appropriately.
    // ... For now tho, we'll assume.
    let mut pg_dump_command = Command::new("pg_dump");

    pg_dump_command.args(&[
            "-h", "postgres",       // The container name
            "-p", "5432",
            "-U", &env::var("POSTGRES_USER").unwrap(),
            "-d", "powerdown_db",     // At time of writing, this is the hardcoded DB name. I should make it an env at some point.
            "-F", "c",               // custom compressed format
            "-f", PG_DUMP_FILENAME,
        ])
        .env("PGPASSWORD", env::var("POSTGRES_PASSWORD").unwrap());

    if let Err(err) = pg_dump_command.output().await {
        eprintln!("[SQL BACKUP] pg_dump failed! Passing err to calling function.");
        return Err(err);
    }

    // DB contents were successfully dumped to `PG_DUMP_FILENAME`, move em to S3 and clean up.

    let s3_client = state.s3_client.clone();

    // TODO: Upload to S3 here.

    // Now delete the file we have on the system.
    if let Err(err) = std::fs::remove_file(PG_DUMP_FILENAME) {
        // If this fails, it doesn't harm the rest of the function. It's just... well, sub-optimal.
        // If it starts causing problems, someone can go in there and delete the file themselves.
        eprintln!("[SQL BACKUP] Removing local file failed! Proceeding as normal. Err: {:?}", err);
    };

    Ok(())
}