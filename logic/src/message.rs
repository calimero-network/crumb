use std::ops::Deref;

use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_storage::collections::UnorderedMap;

use crate::bid::BidId;
use crate::bounty::BountyId;
use crate::types::id::Id;
use crate::user::UserId;

#[derive(
    Eq, Copy, Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
#[borsh(crate = "calimero_sdk::borsh")]
#[serde(crate = "calimero_sdk::serde")]
pub struct MessageId(Id<8, 12>);

impl Deref for MessageId {
    type Target = Id<8, 12>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for MessageId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<[u8; 8]> for MessageId {
    fn from(id: [u8; 8]) -> Self {
        Self(Id::from(id))
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Message {
    pub author: UserId,
    pub timestamp: u64,
    pub target: MessageTarget,
    pub content: String,
    pub reactions: UnorderedMap<UserId, Reaction>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum MessageTarget {
    Bounty(BountyId),
    Bid(BidId),
    Assignment(BidId),
    User(UserId),
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Reaction {
    emoji: String,
    timestamp: u64,
}
