use derive_more::From;
use lib_core::model;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
    // -- RPC
    RpcMethodUnknown(String),
    RpcMissingParams {
        rpc_method: String,
    },
    RpcFailJsonParams {
        rpc_method: String,
    },

    // -- Login
    LoginFailUsernameNotFound,
    LoginFailUserHasNoPwd {
        user_id: i64,
    },
    LoginFailPwdNotMatching {
        user_id: i64,
    },

    // -- Modules
    #[from]
    Model(model::Error),

    // -- External Modules
    #[from]
    SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
