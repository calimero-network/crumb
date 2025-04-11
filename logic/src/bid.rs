use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_storage::collections::Vector;

use crate::assignment::AssignmentId;
use crate::bounty::BountyId;
use crate::message::MessageId;
use crate::types::id::Id;

#[derive(Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
#[serde(crate = "calimero_sdk::serde")]
pub struct BidId(Id<8, 12>);

impl From<[u8; 8]> for BidId {
    fn from(id: [u8; 8]) -> Self {
        Self(Id::from(id))
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Bid {
    pub assignment: Option<AssignmentId>,
    pub brief: String,
    pub bounty: BountyId,
    pub status: BidStatus,
    pub expiry: Option<u64>,
    pub reward: Option<Reward>,
    pub duration: Option<u64>,
    pub comments: Vector<MessageId>,

    pub proposed_at: Option<u64>,
    pub approved_at: Option<u64>,
    pub retracted_at: Option<u64>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Reward {
    pub amount: u128,
    pub account: String,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum BidStatus {
    Proposed,
    Approved,
    Retracted { reason: Option<String> },
}
