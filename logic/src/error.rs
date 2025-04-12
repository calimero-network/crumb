use calimero_sdk::serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(crate = "calimero_sdk::serde")]
#[serde(tag = "kind", content = "data")]
pub enum Error<'a> {
    #[error("user is not registered")]
    NotRegistered,
    #[error("user is already registered")]
    AlreadyRegistered,
    #[error("not found: {0}")]
    NotFound(&'a str),
}
