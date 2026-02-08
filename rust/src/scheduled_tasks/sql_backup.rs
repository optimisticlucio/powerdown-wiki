use std::collections::HashSet;
use std::env;

use chrono::Datelike;
use tokio::process::Command;

use super::get_current_human_readable_time;
use crate::ServerState;

/// The prefix/folder all SQL backups are placed in. Do not put starting nor leading slash.
const S3_SQL_BACKUP_PREFIX: &str = "sql_backups";

/// Runs all processes related to SQL backup, including cleaning up old revisions.
pub async fn run_backup_processes(state: ServerState) {
    println!(
        "[SQL BACKUP PROCESSES] System time is {}, initiating sql backup.",
        get_current_human_readable_time()
    );

    if let Err(err) = backup_db(state.clone()).await {
        eprintln!("[SQL BACKUP PROCESSES] Failed to backup DB! err: {:?}", err);
        return;
    }

    if let Err(err) = clean_up_old_backups(state.clone()).await {
        eprintln!(
            "[SQL BACKUP PROCESSES] Failed to clean old backups! err: {:?}",
            err
        );
        return;
    }

    println!(
        "[SQL BACKUP PROCESSES] System time is {}, sql backup complete.",
        get_current_human_readable_time()
    );
}

/// When run, backs up the entire sql db to the private S3 bucket.
async fn backup_db(state: ServerState) -> Result<(), Box<dyn std::error::Error>> {
    const PG_DUMP_FILENAME: &str = "pg_dump";

    // The command currently assumes the DB is a parallel docker container.
    // If we ever move off of docker, make this command read the ENV for the DB and use it appropriately.
    // ... For now tho, we'll assume.
    let mut pg_dump_command = Command::new("pg_dump");

    pg_dump_command
        .args(&[
            "-h",
            "postgres", // The container name
            "-p",
            "5432",
            "-U",
            &env::var("POSTGRES_USER").unwrap(),
            "-d",
            "powerdown_db", // At time of writing, this is the hardcoded DB name. I should make it an env at some point.
            "-F",
            "c", // custom compressed format
            "-f",
            PG_DUMP_FILENAME,
        ])
        .env("PGPASSWORD", env::var("POSTGRES_PASSWORD").unwrap());

    match pg_dump_command.output().await {
        Err(err) => {
            eprintln!("[SQL BACKUP] pg_dump failed! Passing err to calling function.");
            return Err(Box::new(err));
        }
        Ok(ok) => {
            println!("[SQL BACKUP] pg_dump successful! {:?}", ok);
        }
    }

    // Check filesize of pg_dump, for my own sanity.

    let pg_dump_metadata = std::fs::metadata(PG_DUMP_FILENAME).map_err(|err| {
        eprintln!("[SQL BACKUP] Reading pg_dump metadata failed! Passing err to calling function.");
        err
    })?;

    println!(
        "[SQL BACKUP] pg_dump filesize is {} bytes.",
        pg_dump_metadata.len()
    );

    if pg_dump_metadata.len() == 0 {
        println!("[SQL BACKUP] SQL failed to backup!");
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "pg_dump created empty file",
        )));
    }

    // DB contents were successfully dumped to `PG_DUMP_FILENAME`, move em to S3 and clean up.

    let pg_dump_file = std::fs::read(PG_DUMP_FILENAME).map_err(|err| {
        eprintln!("[SQL BACKUP] Failed to read pg_dump! Passing err upward.");
        Box::new(err)
    })?;

    let pg_dump_target_filename = format!(
        "{}/{}.pgdump",
        S3_SQL_BACKUP_PREFIX,
        get_current_human_readable_time()
    );

    let s3_client = state.s3_client.clone();

    s3_client
        .put_object()
        .bucket(state.config.s3_sql_backup_bucket)
        .key(&pg_dump_target_filename)
        .body(pg_dump_file.into())
        .content_type("application/octet-stream")
        .send()
        .await
        .map_err(|err| {
            eprintln!("[SQL BACKUP] Failed to upload SQL Backup to S3! Passing err upward.");
            Box::new(err)
        })?;

    // Now delete the file we have on the system.
    if let Err(err) = std::fs::remove_file(PG_DUMP_FILENAME) {
        // If this fails, it doesn't harm the rest of the function. It's just... well, sub-optimal.
        // If it starts causing problems, someone can go in there and delete the file themselves.
        eprintln!(
            "[SQL BACKUP] Removing local file failed! Proceeding as normal. Err: {:?}",
            err
        );
    };

    Ok(())
}

/// When run, cleans up old backups from the s3 private bucket, in the following manner:
/// Between today and 3 days ago, all backups are untouched.
/// Between 4 days ago and a week ago, 1 backup is stored per day (the earliest backup is kept each day).
/// Between a week + a day ago and a month ago, 1 backup is stored per week (the sunday backup).
/// From a month + a day ago onward, one backup is kept per month (the earliest sunday).
async fn clean_up_old_backups(state: ServerState) -> Result<(), Box<dyn std::error::Error>> {
    let s3_client = state.s3_client.clone();

    // First, get all of the backups currently on s3.
    let list_objects_output = s3_client.list_objects_v2()
        .bucket(&state.config.s3_sql_backup_bucket)
        .prefix(S3_SQL_BACKUP_PREFIX)
        .send()
        .await
        .map_err(|err| {
            eprintln!("[CLEANUP SQL BACKUPS] Failed getting list of backups on S3! Passing error upwards.");
            Box::new(err)
        })?;

    let list_of_objects = match list_objects_output.contents {
        None => {
            eprintln!("[CLEANUP SQL BACKUPS] No SQL backups found on S3! Finishing operation, but this should not be happening!");
            return Ok(());
        }
        Some(list) => list,
    };

    // Now let's organize the relevant backups into their categories.
    let mut backups_from_4_days_ago_to_week_ago = Vec::new();
    let mut backups_from_over_week_ago_to_month_ago = Vec::new();
    let mut backups_from_over_month_ago = Vec::new();

    let current_timestamp = chrono::Utc::now();
    let three_days_ago = current_timestamp - chrono::Duration::days(3);
    let week_ago = current_timestamp - chrono::Duration::weeks(1);
    let month_ago = current_timestamp - chrono::Duration::days(30);
    let three_months_ago = current_timestamp - chrono::Duration::days(90);

    for backup in list_of_objects {
        // If the last modification date is null, just ignore it for now.
        // We could check the filename later, but I am lazy rn.
        if let Some(last_modification_date) = backup.last_modified {
            let last_modification_date = chrono::DateTime::from_timestamp_millis(
                last_modification_date.to_millis().unwrap(),
            )
            .unwrap();

            match last_modification_date {
                date if date < three_months_ago => {
                    // Old enough that we assume it's already organized.
                }
                date if date < month_ago => {
                    backups_from_over_month_ago.push(backup.key.unwrap());
                }
                date if date < week_ago => {
                    backups_from_over_week_ago_to_month_ago.push(backup.key.unwrap());
                }
                date if date < three_days_ago => {
                    backups_from_4_days_ago_to_week_ago.push(backup.key.unwrap());
                }
                _ => (), // The last 3 days. Leave unmodified.
            }
        }
    }

    // Sort each list for convenience.
    backups_from_4_days_ago_to_week_ago.sort();
    backups_from_over_week_ago_to_month_ago.sort();
    backups_from_over_month_ago.sort();

    // Now let's see which backups we need to delete.
    let mut backup_keys_to_delete: Vec<String> = Vec::new();

    // 4 days to a week: Keep only the earliest backup per day.
    let mut days_already_backed_up: HashSet<String> = HashSet::new();
    // Because the list is sorted, we can assume the first day we see is the oldest.
    for backup_key in backups_from_4_days_ago_to_week_ago {
        let backup_date = backup_key
            .strip_prefix(S3_SQL_BACKUP_PREFIX)
            .unwrap()
            .strip_prefix("/")
            .unwrap()
            .split(" ")
            .next()
            .unwrap(); // Get the YYYY-MM-DD part of the filename.

        if days_already_backed_up.contains(backup_date) {
            backup_keys_to_delete.push(backup_key);
        } else {
            days_already_backed_up.insert(backup_date.to_string());
        }
    }

    // week to a month: only keep sundays.
    for backup_key in backups_from_over_week_ago_to_month_ago {
        let backup_date = backup_key
            .strip_prefix(S3_SQL_BACKUP_PREFIX)
            .unwrap()
            .strip_prefix("/")
            .unwrap()
            .split(" ")
            .next()
            .unwrap(); // Get the YYYY-MM-DD part of the filename.

        let backup_naivedate = chrono::NaiveDate::parse_from_str(backup_date, "%Y-%m-%d").unwrap();

        if backup_naivedate.weekday() != chrono::Weekday::Sun {
            backup_keys_to_delete.push(backup_key);
        }
    }

    // month and above: first sunday of the month
    let mut months_already_backed_up: HashSet<u32> = HashSet::new();
    // Because the list is sorted, we can assume the first day we see is the oldest.
    for backup_key in backups_from_over_month_ago {
        let backup_month = backup_key
            .strip_prefix(S3_SQL_BACKUP_PREFIX)
            .unwrap()
            .strip_prefix("/")
            .unwrap()
            .split(" ")
            .next()
            .unwrap(); // Get the YYYY-MM-DD part of the filename.

        let backup_naivedate = chrono::NaiveDate::parse_from_str(backup_month, "%Y-%m-%d").unwrap();

        let backup_month = backup_naivedate.month();

        if months_already_backed_up.contains(&backup_month) {
            backup_keys_to_delete.push(backup_key);
        } else {
            months_already_backed_up.insert(backup_month);
        }
    }

    // Now that we have the keys to delete, nuke em.
    if backup_keys_to_delete.is_empty() {
        return Ok(());
    }

    let _ = crate::utils::delete_keys_from_s3(
        &s3_client,
        &state.config.s3_sql_backup_bucket,
        &backup_keys_to_delete.into(),
    )
    .await
    .map_err(|err| {
        eprintln!(
            "[CLEANUP SQL BACKUPS] Failed deleting old backups! err: {}",
            err
        );
    });

    Ok(())
}
