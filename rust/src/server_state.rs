use std::env;

use deadpool::managed::Pool;
use deadpool_postgres::{self, Manager, ManagerConfig, RecyclingMethod, Runtime};
use postgres::NoTls;
use tokio::join;

#[derive(Clone)]
pub struct ServerState {
    db_pool: Pool<Manager>
}

impl ServerState {
    /// Returns a ServerState with default initializations.
    pub async fn initalize() -> Self {
        let (db_pool,) = join!(Self::initialize_db());

        ServerState {
            db_pool
        }
    }

    async fn initialize_db() -> Pool<Manager>{
        let mut db_config = deadpool_postgres::Config::new();

        db_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        db_config.dbname = Some("powerdown_db".to_owned()); // Hardcoded in docker-compose
        db_config.host = Some("postgres".to_owned()); // The postgres docker container's name. 
        db_config.password = env::var("POSTGRES_PASSWORD").ok();

        let db_pool = db_config.create_pool(Some(deadpool::Runtime::Tokio1), NoTls).unwrap();

        db_pool
    }
}