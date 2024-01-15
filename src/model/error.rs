use crate::model::store;
use crate::pwd;
use derive_more::From;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
	EntityNotFound {
		entity: &'static str,
		id: i64,
	},
	ListLimitOverMax {
		max: i64,
		actual: i64,
	},

	// -- Modules
	#[from]
	Pwd(pwd::Error),
	#[from]
	Store(store::Error),

    // -- Externals
    #[from]
	Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
	#[from]
	SeaQuery(#[serde_as(as = "DisplayFromStr")] sea_query::error::Error),
	#[from]
	ModqlIntoSea(#[serde_as(as = "DisplayFromStr")] modql::filter::IntoSeaError),

    MongoEntityNotFound { entity: &'static str, id: String },
    MongoInvalidIDError(String),
    MongoDuplicateError(String),
    MongoQueryError(String),
}

// region:    --- Froms

impl From<mongodb::error::Error> for Error {
    fn from(error: mongodb::error::Error) -> Self {
        if error
            .to_string()
            .contains("E11000 duplicate key error collection")
        {
            return Error::MongoDuplicateError("Duplicate key".to_string());
        }

        Error::MongoQueryError("There was an error".to_string())
    }
}

// endregion: --- Froms

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
