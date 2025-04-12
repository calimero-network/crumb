use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_storage::collections::UnorderedSet;

use crate::assignment::AssignmentId;
use crate::bounty::BountyId;
use crate::message::MessageId;
use crate::types::id;

id::define!(pub BidId<8, 12>);

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
    pub comments: UnorderedSet<MessageId>,

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
