use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_storage::collections::{UnorderedMap, UnorderedSet};

use crate::bid::BidId;
use crate::bounty::BountyId;
use crate::message::MessageId;
use crate::types::id;
use crate::user::UserId;

id::define!(pub AssignmentId<8, 12>);

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Assignment {
    pub assignee: UserId,
    pub bounty: BountyId,

    pub message: MessageId,
    pub bid: Option<BidId>,
    pub links: UnorderedSet<String>,

    pub status: AssignmentStatus,
    pub expiry: Option<u64>,
    pub reward: UnorderedMap<String, u128>,
    pub duration: Option<u64>,

    pub assigned_at: Option<u64>,
    pub accepted_at: Option<u64>,
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
