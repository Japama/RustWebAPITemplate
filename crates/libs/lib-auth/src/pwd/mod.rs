// region: -- Modules

mod error;
mod scheme;

use std::str::FromStr;
use lazy_regex::regex_captures;
pub use self::error::{Error, Result};

use crate::pwd::scheme::{DEFAULT_SCHEME, get_scheme, SchemeStatus};
use uuid::Uuid;
// endregion: -- Modules

// region: Types

pub struct ContentToHash {
    pub content: String,
    pub salt: Uuid,
}

// endregion: Types

// region: Public Functions

/// Hash the password with the default scheme.
pub fn hash_pwd(to_hash: &ContentToHash) -> Result<String> {
    hash_for_scheme(DEFAULT_SCHEME, to_hash)
}

/// Validate if an ContentToHash matches.
pub fn validate_pwd(to_hash: &ContentToHash, pwd_ref: &str) -> Result<SchemeStatus> {
    let PwdParts{
        scheme_name,
        hashed
    } = pwd_ref.parse()?;

    validate_for_scheme(&scheme_name, to_hash, &hashed)?;

    if scheme_name == DEFAULT_SCHEME{
        Ok(SchemeStatus::Ok)
    } else{
        Ok(SchemeStatus::Outdated)
    }
}

// endregion: Public Functions

// region: Privates

fn hash_for_scheme(scheme_name: &str, to_hash: &ContentToHash) -> Result<String> {
    let pwd_hashed = get_scheme(scheme_name)?.hash(to_hash)?;
    Ok(format!("#{scheme_name}#{pwd_hashed}"))
}

fn validate_for_scheme(
    scheme_name: &str,
    to_hash: &ContentToHash,
    pwd_ref: &str
) -> Result<()> {
    get_scheme(scheme_name)?.validate(to_hash, pwd_ref)?;
    Ok(())
}

struct PwdParts{
    /// The scheme only
    scheme_name: String,
    /// The hashed password
    hashed: String,
}

impl FromStr for PwdParts{
    type Err = Error;

    fn from_str(pwd_with_scheme: &str) -> Result<Self> {
        regex_captures!(
            r#"^#(\w+)#(.*)"#, // a literal regex
            pwd_with_scheme
        )
            .map(|(_, scheme, hashed)| Self{
                scheme_name: scheme.to_string(),
                hashed:     hashed.to_string()
            })
            .ok_or(Error::PwdWithSchemeFailedParse)
    } 
}

// endregion: Privates

// region: Tests

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_multi_scheme_ok() -> Result<()> {
        // Setup & Fixtures
        let fx_salt = Uuid::parse_str("f05e8961-d6ad-4086-9e78-a6de065e5453")?;
        let fx_to_hash = ContentToHash {
            content: "hello wolrd".to_string(),
            salt: fx_salt,
        };

        // Exec
        let pwd_hashed = hash_pwd(&fx_to_hash)?;
        println!("->> pwd_hashed: {pwd_hashed}");
        let pwd_validate = validate_pwd(&fx_to_hash, &pwd_hashed)?;
        println!("->>   validate: {pwd_validate:?}");

        Ok(())
    }
}

// endregion: Tests
