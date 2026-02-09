//! # Scheduled Tasks
//!
//! `scheduled_tasks` is the module handling all the tasks that the server needs to regularly perform,
//! outside of the context of a given server request.
//!
//! The `initiate(ServerState)` function is the one the main function should call to initiate all the relevant tasks.

use crate::{scheduled_tasks::lib::clean_temp_db_entries, ServerState};
use tokio_cron_scheduler::{Job, JobScheduler};

mod lib;
mod sql_backup;

pub use lib::get_current_human_readable_time;
pub use sql_backup::run_backup_processes;

/// Sets up all the periodic tasks that the server needs to do, so they'll run at the appropriate times.
pub async fn initiate_scheduled_tasks(state: ServerState) {
    // This section uses `tokio_cron_scheduler` to make sure that these tasks occur consistently on given
    // times of day, so that restarting the server doesn't restart the clock on when a given task should
    // be run. The syntax here is from the `croner` library so you can check it aswell for info,
    // but in short:
    //
    // [seconds] [minute] [hour] [day of month] [month] [day of week]
    // - * is a wildcard, meaning any value applies
    // - (*/num) means every value that cleanly divides by num. For example, (*/15) is once every 15 min.
    // - There's some shorthands, for example @hourly and @daily, that replace this entire pattern.

    let job_scheduler = JobScheduler::new()
        .await
        .expect("[JOB SCHEDULER] Failed creating job scheduler!");

    // I tried making this iterate over a list of tasks, and every time I ran into some other bug.
    // If you can fix it, PLEASE DO. This is nigh unreadable.

    // Clean temp db entries once an hour.
    let cloned_state = state.clone();

    job_scheduler
        .add(
            Job::new_async("@hourly", move |_uuid, _scheduler| {
                let state = cloned_state.clone();
                Box::pin(async move {
                    clean_temp_db_entries(&state).await;
                })
            })
            .inspect_err(|err| {
                eprintln!("[JOB SCHEDULER] Failed creating clean_temp_db_entries job: {err:?}")
            })
            .unwrap(),
        )
        .await
        .inspect_err(|err| {
            eprintln!("[JOB SCHEDULER] Failed adding clean_temp_db_entries job to list: {err:?}")
        })
        .unwrap();

    // Backup DB once a day.
    let cloned_state = state.clone();

    job_scheduler
        .add(
            Job::new_async("@daily", move |_uuid, _scheduler| {
                let state = cloned_state.clone();
                Box::pin(async move {
                    sql_backup::run_backup_processes(&state).await;
                })
            })
            .inspect_err(|err| {
                eprintln!("[JOB SCHEDULER] Failed creating run_backup_processes job: {err:?}")
            })
            .unwrap(),
        )
        .await
        .inspect_err(|err| {
            eprintln!("[JOB SCHEDULER] Failed adding run_backup_processes job to list: {err:?}")
        })
        .unwrap();

    job_scheduler
        .start()
        .await
        .inspect_err(|err| eprintln!("[JOB SCHEDULER] Failed starting job scheduler: {err:?}"))
        .unwrap();
}
