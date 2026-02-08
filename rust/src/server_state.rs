use std::env;

use deadpool::managed::Pool;
use deadpool_postgres::{self, Manager, ManagerConfig, RecyclingMethod};
use postgres::NoTls;
use tokio::join;

pub mod config;

#[derive(Debug, Clone)]
pub struct ServerState {
    pub db_pool: Pool<Manager>,
    pub s3_client: aws_sdk_s3::Client, // Apparently cloning these doesn't cause race conditions.
    // If this ^ ends up being a bottleneck, create a client pool with deadpool.
    pub config: config::Config,
}

impl ServerState {
    /// Returns a ServerState with default initializations.
    /// Will panic if any ENV variables are missing to ensure there isn't buggy runtime behaviour.
    pub async fn initalize() -> Self {
        let (db_pool, s3_client) = join!(Self::initialize_db(), Self::initialize_s3_connection());

        let config = config::Config::initialize();

        ServerState {
            db_pool,
            s3_client,
            config,
        }
    }

    async fn initialize_db() -> Pool<Manager> {
        let mut db_config = deadpool_postgres::Config::new();

        db_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        db_config.dbname = env::var("POSTGRES_DATABASE").ok();
        db_config.host = env::var("POSTGRES_HOST").ok();
        db_config.password = env::var("POSTGRES_PASSWORD").ok();
        db_config.user = env::var("POSTGRES_USER").ok();

        let db_pool = db_config
            .create_pool(Some(deadpool::Runtime::Tokio1), NoTls)
            .unwrap();

        db_pool
    }

    async fn initialize_s3_connection() -> aws_sdk_s3::Client {
        let sdk_config = aws_config::load_from_env().await;

        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config).build();

        let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

        s3_client
    }
}
