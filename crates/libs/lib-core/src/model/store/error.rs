use mongodb::error::Error as MongoError;
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    FailToCreatePool(String),
    FailUriMongoDB(String),
}

impl From<MongoError> for Error {
    fn from(error: MongoError) -> Self {
        Error::FailUriMongoDB(format!("MongoDB error: {:?}", error))
    }
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
