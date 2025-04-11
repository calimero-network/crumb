use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_storage::collections::{UnorderedMap, Vector};

use crate::assignment::AssignmentId;
use crate::bid::BidId;
use crate::message::MessageId;
use crate::types::id::Id;
use crate::user::UserId;
use crate::LabelId;

#[derive(Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
#[serde(crate = "calimero_sdk::serde")]
pub struct BountyId(Id<8, 12>);

impl From<[u8; 8]> for BountyId {
    fn from(id: [u8; 8]) -> Self {
        Self(Id::from(id))
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Bounty {
    pub is_epic: bool,
    pub author: String,
    pub description: String,
    pub reviewers: Vector<UserId>,
    pub labels: Vector<LabelId>,
    pub award: Option<u128>,
    pub bids: UnorderedMap<UserId, BidId>,
    pub assignment: UnorderedMap<UserId, AssignmentId>,
    pub status: BountyStatus,
    pub deadline: Option<u64>,
    pub parent: Option<BountyId>,
    pub children: Vector<BountyId>,
    pub comments: Vector<MessageId>,

    pub triaged_by: Option<UserId>,
    pub approved_by: Option<UserId>,
    pub closed_by: Option<UserId>,

    pub proposed_at: Option<u64>,
    pub triaged_at: Option<u64>,
    pub approved_at: Option<u64>,
    pub closed_at: Option<u64>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum BountyStatus {
    Proposed,
    Triaged,
    Approved,
    Closed { reason: ClosureReason },
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum ClosureReason {
    Completed { assignment: AssignmentId },
    Abandoned,
    Expired,
}
