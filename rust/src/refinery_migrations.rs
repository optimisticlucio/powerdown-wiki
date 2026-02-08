//! # Refinery Migrations
//!
//! This section handles SQL migrations using the Refinery crate.
//! To add a new migration, create a new file in the /rust/refinery_migrations directory with the naming scheme:
//! V[YYYYMMMDDHHmm]__[description].sql
//!
//! For example, if you made a migration on Dec 24th, 2021, at 10:41, to give users profile pics, you may name it
//! V202112241041__profilepics.sql
//!
//!
//! The few files at the top which violate this naming scheme are the files initially used to create the DB.
use std::ops::DerefMut;

use crate::ServerState;
use refinery::embed_migrations;

embed_migrations!("./refinery_migrations");

/// Runs the Refinery migrator on the sql db.
pub async fn run_migrations(state: &ServerState) -> Result<refinery::Report, refinery::Error> {
    let mut db_connection = state.db_pool.get().await.unwrap();
    let migration_connection = db_connection.deref_mut().deref_mut();

    migrations::runner().run_async(migration_connection).await
}
