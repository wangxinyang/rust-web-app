mod error;

use crate::config;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;

pub use self::error::{Error, Result};

pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> Result<Db> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(500))
        .connect(&config().DB_URL)
        .await
        .map_err(|ex| Error::FailToCreatePool(ex.to_string()))
}
