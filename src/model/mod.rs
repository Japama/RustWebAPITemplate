/* Model Layer
    Design:

    - The model layer normalizes the application's data type structures and access.
    - All application code data access must go through the Model layer.
    - The `ModelManager` holds the internal states/resources needed by Model Controllers to access data. (e.g., db_pool, S3 client, redis client).
    - Model Controllers (e.g., `TaskBmc`, `ProjectBmc`) implement CRUD and other data access methods pon a given "entity" (e.g., `Task`, `Project`). (`Bmc` is short for Backend Model Controller).
    - In frameworks like Axum, Tauri, `ModelManager`are typically used as App State.
    - Model Manager are designed to be Passed as an argument to all Model Controllers functions.
*/

// region: --- Modules

mod base;
mod error;
mod store;
pub mod task;

pub use self::error::{Error, Result};
use crate::model::store::{new_db_pool, Db};

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    /// Constructor
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;

        Ok(ModelManager { db })
    }

    // Returns the sqlx db pool reference.
    // Only for the model layer
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}

// endregion: --- Modules
