mod error;
mod store;
mod task;

pub use self::error::{Error, Result};

use self::store::Db;

// endregion: --- Modules

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        // ? mark需要实现From<Store::Error> for Model::Error
        let db = store::new_db_pool().await?;
        Ok(ModelManager { db })
    }

    // returns the sqlx db pool reference
    // Only for the model layer
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
