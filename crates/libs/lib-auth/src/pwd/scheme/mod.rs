// region: Modules

mod error;
mod scheme_01;

pub use self::error::{Error, Result};

use crate::pwd::ContentToHash;

// endregion: Modules

// region: Trait

pub trait Scheme {
    fn hash(&self, to_hash: &ContentToHash) -> Result<String>;

    fn validate(&self, to_hash: &ContentToHash, pwd_ef: &str) -> Result<()>;
}

pub enum SchemeStatus {
    Ok,       // The pwd uses the latest scheme. All good.
    Outdated, // The pwd uses an old scheme.
}

pub fn get_scheme(scheme_name: &str) -> Result<Box<dyn Scheme>> {
    match scheme_name {
        "01" => Ok(Box::new(scheme_01::Scheme01)),

        _ => Err(Error::SchemeNotFound(scheme_name.to_string())),
    }
}

// endregion: Trait
