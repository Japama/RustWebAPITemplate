use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    EntityNotFound { entity: &'static str, id: i64 },
    // -- Modules
    // Crypt(crypt::Error),
    // Store(store::Error),

    // -- Externals
    // Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
}
