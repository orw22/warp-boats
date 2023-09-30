use std::sync::Arc;

use anyhow::Result;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    SqliteConnection,
};
use tokio::sync::Mutex;

pub struct ConnectionPool(Pool<ConnectionManager<SqliteConnection>>);

impl ConnectionPool {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create connection pool");
        Self(pool)
    }

    pub fn acquire(&self) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>> {
        Ok(self.0.get()?)
    }
}

pub type SharedConnectionPool = Arc<Mutex<ConnectionPool>>;
