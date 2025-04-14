use std::collections::BTreeMap;

use calimero_sdk::app;
use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_storage::collections::UnorderedMap;
use thiserror::Error;

use crate::assignment::AssignmentId;
use crate::bounty::BountyId;
use crate::message::{MessageId, MessageTarget};
use crate::types::id::{self, IdExt};
use crate::user::UserId;
use crate::utils::unique;
use crate::AppState;

id::define!(pub BidId<8, 12>);

const MAX_BID_BRIEF_LENGTH: usize = 2_000;
const MAX_BID_REWARD_RECIPIENTS: usize = 50;

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

#[derive(Debug, Error, Serialize)]
#[serde(crate = "calimero_sdk::serde")]
#[serde(tag = "kind", content = "data")]
pub enum Error {
    #[error("brief too long ({got} > {max})")]
    BriefTooLong { got: usize, max: usize },
    #[error("too many reward recipients ({got} > {max})")]
    TooManyRewardRecipients { got: usize, max: usize },
}

fn validate_bid_brief(brief: &str) -> app::Result<()> {
    if brief.len() > MAX_BID_BRIEF_LENGTH {
        app::bail!(Error::BriefTooLong {
            got: brief.len(),
            max: MAX_BID_BRIEF_LENGTH,
        });
    }

    Ok(())
}

fn validate_bid_reward(reward_count: usize) -> app::Result<()> {
    if reward_count > MAX_BID_REWARD_RECIPIENTS {
        app::bail!(Error::TooManyRewardRecipients {
            got: reward_count,
            max: MAX_BID_REWARD_RECIPIENTS,
        });
    }

    Ok(())
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

        validate_bid_brief(&request.brief)?;
        validate_bid_reward(request.reward.len())?;

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
