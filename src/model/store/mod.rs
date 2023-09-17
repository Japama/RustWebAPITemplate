// region:    --- Modules

mod error;

pub use self::error::{Error, Result};
use mongodb::options::ClientOptions;
use mongodb::Client;

use crate::config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

// endregion: --- Modules

// region: Postgres
pub type PostgresDb = Pool<Postgres>;

pub async fn new_db_pool() -> Result<PostgresDb> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config().DB_URL)
        .await
        .map_err(|ex| Error::FailToCreatePool(ex.to_string()))
}
// endregion: Postgres

// region: MongoDB
pub type MongoDb = Client;

pub async fn new_mongo_client() -> Result<Client> {
    let mut client_options = ClientOptions::parse(&config().MONGO_DB_URL).await?;
    let client = Client::with_options(client_options)?;
    Ok(client)
}
// endregion: MongoDB
