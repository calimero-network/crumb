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

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct CreateBidRequest {
    pub brief: String,
    pub bounty: BountyId,
    pub expiry: Option<u64>,
    pub reward: BTreeMap<String, u128>,
    pub duration: Option<u64>,
}

#[app::logic]
impl AppState {
    pub fn create_bid(&mut self, request: CreateBidRequest) -> app::Result<BidId> {
        let user_id = self.current_user();

        let mut user = self.get_registered_user(&user_id)?;

        let bid_id = unique(|| BidId::random(), |id| self.bids.contains(id))?;

        let message_id = self.internal_post_message(
            user_id,
            &mut user,
            MessageTarget::Bid(bid_id),
            request.brief,
        )?;

        let bid = Bid {
            author: user_id,
            bounty: request.bounty,

            message: message_id,

            assignment: None,

            status: BidStatus::Proposed,
            expiry: request.expiry,
            reward: request.reward.into_iter().collect(),
            duration: request.duration,

            proposed_at: None,
            approved_at: None,
            retracted_at: None,
        };

        let _ignored = user.bids.insert(bid_id)?;

        let _ignored = self.users.insert(user_id, user)?;
        let _ignored = self.bids.insert(bid_id, bid)?;

        Ok(bid_id)
    }
}
