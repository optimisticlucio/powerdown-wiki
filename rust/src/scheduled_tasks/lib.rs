use crate::ServerState;

// Cleans all seemingly-orphaned temp db entries, like a failed file upload or somesuch.
pub async fn clean_temp_db_entries(state: ServerState) {
    println!("[CLEAN TEMP DB ENTRIES] System time is {}, cleaning up old DB entries.", get_current_human_readable_time());

    let mut entries_cleaned_up = 0;
    // TODO: Implement

    println!("[CLEAN TEMP DB ENTRIES] Cleanup complete, {} entries removed.", entries_cleaned_up);
}

// Returns the current UTC time and date in a human-readable format.
pub fn get_current_human_readable_time() -> String {
    chrono::Utc::now().format("%d/%m/%Y %T").to_string()
}