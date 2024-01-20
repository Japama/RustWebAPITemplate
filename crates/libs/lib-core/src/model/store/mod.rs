// region:    --- Modules

use mongodb::options::ClientOptions;
use mongodb::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub use self::error::{Error, Result};
use crate::core_config;

mod error;

// endregion: --- Modules

// region: Postgres
pub type PostgresDb = Pool<Postgres>;

pub async fn new_db_pool() -> Result<PostgresDb> {
    // * See NOTE 1) below
    let max_connections = if cfg!(test) { 1 } else { 5 };

    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&core_config().DB_URL)
        .await
        .map_err(|ex| Error::FailToCreatePool(ex.to_string()))
}

// NOTE 1) This is not an ideal situation; however, with sqlx 0.7.1, when executing `cargo test`, some tests that use sqlx fail at a
//         rather low level (in the tokio scheduler). It appears to be a low-level thread/async issue, as removing/adding
//         tests causes different tests to fail. The cause remains uncertain, but setting max_connections to 1 resolves the issue.
//         The good news is that max_connections still function normally for a regular run.
//         This issue is likely due to the unique requirements unit tests impose on their execution, and therefore,
//         while not ideal, it should serve as an acceptable temporary solution.
//         It's a very challenging issue to investigate and narrow down. The alternative would have been to stick with sqlx 0.6.x, which
//         is potentially less ideal and might lead to confusion as to why we are maintaining the older version in this blueprint.

// endregion: Postgres

// region: MongoDB
pub type MongoDb = Client;

pub async fn new_mongo_client() -> Result<Client> {
    let client_options = ClientOptions::parse(&core_config().MONGO_DB_URL).await?;
    let client = Client::with_options(client_options)?;
    Ok(client)
}
// endregion: MongoDB
