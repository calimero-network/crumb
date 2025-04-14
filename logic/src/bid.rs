use std::collections::BTreeMap;

use calimero_sdk::app;
use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_storage::collections::UnorderedMap;

use crate::assignment::AssignmentId;
use crate::bounty::BountyId;
use crate::message::{MessageId, MessageTarget};
use crate::types::id::{self, IdExt};
use crate::user::UserId;
use crate::utils::unique;
use crate::AppState;

id::define!(pub BidId<8, 12>);

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Bid {
    pub author: UserId,
    pub bounty: BountyId,

    pub message: MessageId,
    pub assignment: Option<AssignmentId>,

    pub status: BidStatus,
    pub expiry: Option<u64>,
    pub reward: UnorderedMap<String, u128>,
    pub duration: Option<u64>,

    pub proposed_at: Option<u64>,
    pub approved_at: Option<u64>,
    pub retracted_at: Option<u64>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum BidStatus {
    Proposed,
    Approved,
    Retracted { reason: Option<String> },
}
