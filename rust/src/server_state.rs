use std::env;

use deadpool::managed::Pool;
use deadpool_postgres::{self, Manager, ManagerConfig, RecyclingMethod};
use postgres::NoTls;
use tokio::join;

#[derive(Clone)]
pub struct ServerState {
    pub db_pool: Pool<Manager>,
    pub s3_client: aws_sdk_s3::Client, // Apparently cloning these doesn't cause race conditions. 
    // If this ^ ends up being a bottleneck, create a client pool with deadpool.

    // You have to pass the bucket names around unless you want to make a wrapper struct for buckets,
    // which, frankly, I don't want to do.
    pub s3_public_bucket: String, // The name of the public bucket, passed from env.
    pub s3_sql_backup_bucket: String, // The name of the sql backup bucket, passed from env.
}

impl ServerState {
    /// Returns a ServerState with default initializations.
    /// Will panic if any ENV variables are missing to ensure there isn't buggy runtime behaviour.
    pub async fn initalize() -> Self {
        let (db_pool, s3_client) = join!(Self::initialize_db(), Self::initialize_s3_connection());

        let s3_public_bucket = env::var("S3_PUBLIC_BUCKET_NAME").unwrap(); 
        let s3_sql_backup_bucket = env::var("S3_SQL_BACKUP_BUCKET_NAME").unwrap();

        ServerState {
            db_pool,
            s3_client,
            s3_public_bucket,
            s3_sql_backup_bucket
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
        db_config.user = env::var("POSTGRES_USER").ok();

        let db_pool = db_config.create_pool(Some(deadpool::Runtime::Tokio1), NoTls).unwrap();

        db_pool
    }

    async fn initialize_s3_connection() -> aws_sdk_s3::Client {
        let sdk_config = aws_config::load_from_env().await;

        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(true) // Comment this out once live! This is only for debugging!
            .build();

        let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

        s3_client
    }
}