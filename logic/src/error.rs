use calimero_sdk::serde::Serialize;
use calimero_storage::collections::StoreError;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(crate = "calimero_sdk::serde")]
#[serde(tag = "kind", content = "data")]
pub enum Error<'a> {
    #[error("not found: {0}")]
    NotFound(&'a str),
    #[error("store error: {0}")]
    StoreError(#[from] StoreError),
}
