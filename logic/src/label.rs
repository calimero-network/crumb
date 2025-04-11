use std::ops::Deref;

use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};

use crate::types::id::Id;

#[derive(
    Eq, Copy, Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
#[borsh(crate = "calimero_sdk::borsh")]
#[serde(crate = "calimero_sdk::serde")]
pub struct LabelId(Id<8, 12>);

impl Deref for LabelId {
    type Target = Id<8, 12>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for LabelId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<[u8; 8]> for LabelId {
    fn from(id: [u8; 8]) -> Self {
        Self(Id::from(id))
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
#[serde(crate = "calimero_sdk::serde")]
pub struct Label {
    pub name: String,
}
