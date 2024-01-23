// region:    --- Modules

mod error;
mod params;
mod params_mongo;
mod resources;
pub mod router;
mod rpcs;

pub use self::error::{Error, Result};
pub use params::*;
pub use resources::RpcResources;
pub use router::RpcRequest;

pub use rpcs::*;

// endregion: --- Modules
