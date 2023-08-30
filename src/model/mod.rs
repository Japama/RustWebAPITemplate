mod error;

pub use self::error::{Error, Result};

// endregion: --- Modules

#[derive(Clone)]
pub struct ModelManager {}

impl ModelManager {
    /// Constructor
    pub async fn new() -> Result<Self> {
        Ok(ModelManager {})
    }
}
