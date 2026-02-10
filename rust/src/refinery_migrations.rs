//! # Refinery Migrations
//!
//! This section handles SQL migrations using the Refinery crate.
//! To add a new migration, create a new file in the /rust/refinery_migrations directory with the naming scheme:
//! V[num]__[description].sql
//!
//! The migrations have to be sequential because... well, I accidentally set it to be sequential and it's too late to back out now.
//! Thankfully, the sequential migrations force any pull requests to be fully up-to-date with main,
//! so there's a much lower likelyhood we accidentally fuck something up with these migrations. So, yay?
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
