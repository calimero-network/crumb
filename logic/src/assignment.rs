use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_storage::collections::UnorderedSet;

use crate::bid::BidId;
use crate::bounty::BountyId;
use crate::message::MessageId;
use crate::types::id;

id::define!(pub AssignmentId<8, 12>);

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Assignment {
    pub bid: Option<BidId>,
    pub bounty: BountyId,
    pub links: UnorderedSet<String>,
    pub status: AssignmentStatus,
    pub expiry: Option<u64>,
    pub comments: UnorderedSet<MessageId>,

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
