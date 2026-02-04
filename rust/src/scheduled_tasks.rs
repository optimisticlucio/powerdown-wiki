//! # Scheduled Tasks
//! 
//! `scheduled_tasks` is the module handling all the tasks that the server needs to regularly perform,
//! outside of the context of a given server request.
//! 
//! The `initiate(ServerState)` function is the one the main function should call to initiate all the relevant tasks.

use tokio::time::{interval, Duration};
use crate::{ServerState, scheduled_tasks::lib::clean_temp_db_entries};

mod sql_backup;
mod lib;

pub use lib::get_current_human_readable_time;

/// Sets up all the periodic tasks that the server needs to do, so they'll run at the appropriate times.
pub fn initiate_scheduled_tasks(state: ServerState) {
    // Durations frequently used, for convenience.
    const MINUTE_DURATION: Duration = Duration::from_secs(60);
    const HOUR_DURATION: Duration = MINUTE_DURATION.saturating_mul(60);
    const DAY_DURATION: Duration = HOUR_DURATION.saturating_mul(24);

    // Ok so - I tried making this neater with like, a list and such,
    // but I was running into headaches with types and boxing and such.
    // This may be repetitive, but it's the easiest and least computationally
    // expensive way to run all these functions that I've seen so far.
    // If you know of a better way, PLEASE.

    tokio::spawn(run_periodically(state.clone(), sql_backup::run_backup_processes, DAY_DURATION.checked_div(2).unwrap()));
    tokio::spawn(run_periodically(state.clone(), clean_temp_db_entries, HOUR_DURATION));
}

/// Given a certain task to perform, and how often to perform it, repeatedly calls this task every `frequency`.
/// The task in question is assumed to need the ServerState struct.
async fn run_periodically<F, Fut>(state: ServerState, task: F, frequency: Duration)
where
    F: Fn(ServerState) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut interval = interval(frequency);
    interval.tick().await; // Skip the instant first trigger.
    loop {
        interval.tick().await;
        task(state.clone()).await;
    }
}

