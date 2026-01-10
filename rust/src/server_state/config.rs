use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // You have to pass the bucket names around unless you want to make a wrapper struct for buckets,
    // which, frankly, I don't want to do.
    pub s3_public_bucket: String, // The name of the public bucket, passed from env.
    pub s3_sql_backup_bucket: String, // The name of the sql backup bucket, passed from env.
}

impl Config {
    /// Gets the relevant data from ENV and returns a new instance of Config.
    pub fn initialize() -> Self {
        let s3_public_bucket = env::var("S3_PUBLIC_BUCKET_NAME").unwrap(); 
        let s3_sql_backup_bucket = env::var("S3_SQL_BACKUP_BUCKET_NAME").unwrap();

        Self {
            s3_public_bucket,
            s3_sql_backup_bucket
        }
    }
}