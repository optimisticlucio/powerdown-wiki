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

        // TODO: Actually hook up to the stupid db.
        db_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        // TODO: Settings!

        let db_pool = db_config.create_pool(Some(deadpool::Runtime::Tokio1), NoTls).unwrap();

        // TODO: I think this part is just for testing and I can remove it?
        for i in 1..10i32 {
            let client = db_pool.get().await.unwrap();
            let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
            let rows = client.query(&stmt, &[&i]).await.unwrap();
            let value: i32 = rows[0].get(0);
            assert_eq!(value, i + 1);
        }

        db_pool
    }
}