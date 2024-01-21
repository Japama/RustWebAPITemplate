// region: -- Modules

mod error;
mod scheme;

pub use self::error::{Error, Result};

use crate::pwd::scheme::get_scheme;
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
    // FIXME
    Ok("FIXME hash_pwd".to_string())
}

/// Validate if an ContentToHash matches.
pub fn validate_pwd(to_has: &ContentToHash, pwd_ref: &str) -> Result<String> {
    // FIXME
    Ok("FUXNE validate_pwd".to_string())
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

struct PwdParts

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
