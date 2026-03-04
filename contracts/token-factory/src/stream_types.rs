use soroban_sdk::{contracttype, Address, String};
use crate::types::Error;

/// Stream information with optional metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamInfo {
    pub id: u32,
    pub creator: Address,
    pub recipient: Address,
    pub amount: i128,
    pub metadata: Option<String>,
    pub created_at: u64,
}

/// Validate stream metadata length (max 512 chars)
pub fn validate_metadata(metadata: &Option<String>) -> Result<(), Error> {
    if let Some(meta) = metadata {
        let len = meta.len();
        if len == 0 || len > 512 {
            return Err(Error::InvalidParameters);
        }
    }
    Ok(())
}
