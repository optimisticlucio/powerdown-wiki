use crate::ServerState;

// Cleans all seemingly-orphaned temp db entries, like a failed file upload or somesuch.
pub async fn clean_temp_db_entries(state: &ServerState) {
    println!(
        "[CLEAN TEMP DB ENTRIES] System time is {}, cleaning up old DB entries.",
        get_current_human_readable_time()
    );

    let db_connection = match state.db_pool.get().await {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("[CLEAN TEMP DB ENTRIES] Failed to get sql connection! {err:?}");
            return;
        }
    };

    // Clean up anything that's been processing for over 10 minutes.
    const ART_DB_CLEANUP_QUERY: &str = "DELETE FROM art WHERE post_state = 'processing' AND last_modified_date < NOW() - INTERVAL '10 minutes'";
    let art_db_cleanup_rows_modified = match db_connection.execute(ART_DB_CLEANUP_QUERY, &[]).await
    {
        Ok(rows_modified) => rows_modified,
        Err(err) => {
            eprintln!(
                "[CLEAN TEMP DB ENTRIES] Failed to clean up art db! {err:?}. Continuing cleanup."
            );
            0
        }
    };

    const CHARACTER_DB_CLEANUP_QUERY: &str = "DELETE FROM character WHERE post_state = 'processing' AND last_modified_date < NOW() - INTERVAL '10 minutes'";
    let character_db_cleanup_rows_modified = match db_connection
        .execute(CHARACTER_DB_CLEANUP_QUERY, &[])
        .await
    {
        Ok(rows_modified) => rows_modified,
        Err(err) => {
            eprintln!("[CLEAN TEMP DB ENTRIES] Failed to clean up character db! {err:?}. Continuing cleanup.");
            0
        }
    };

    let entries_cleaned_up = art_db_cleanup_rows_modified + character_db_cleanup_rows_modified;

    println!("[CLEAN TEMP DB ENTRIES] Cleanup complete, {entries_cleaned_up} entries removed.");
}

// Returns the current UTC time and date in a human-readable format.
pub fn get_current_human_readable_time() -> String {
    chrono::Utc::now().format("%Y-%m-%d %T").to_string()
}
