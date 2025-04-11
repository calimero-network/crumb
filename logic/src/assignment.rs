use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_storage::collections::Vector;

use crate::bid::BidId;
use crate::bounty::BountyId;
use crate::message::MessageId;
use crate::types::id::Id;

#[derive(Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
#[serde(crate = "calimero_sdk::serde")]
pub struct AssignmentId(Id<8, 12>);

impl From<[u8; 8]> for AssignmentId {
    fn from(id: [u8; 8]) -> Self {
        Self(Id::from(id))
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Assignment {
    pub bid: Option<BidId>,
    pub bounty: BountyId,
    pub links: Vector<String>,
    pub status: AssignmentStatus,
    pub expiry: Option<u64>,
    pub comments: Vector<MessageId>,

    pub received_at: Option<u64>,
    pub in_progress_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub abandoned_at: Option<u64>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum AssignmentStatus {
    Received,
    InProgress,
    Completed,
    Abandoned { reason: Option<String> },
}
