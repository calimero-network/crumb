use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_sdk::{app, env};
use calimero_storage::collections::{UnorderedMap, UnorderedSet};
use thiserror::Error;

use crate::assignment::AssignmentId;
use crate::bid::BidId;
use crate::bounty::BountyId;
use crate::types::id::{self, IdExt};
use crate::user::{User, UserId};
use crate::utils::unique;
use crate::AppState;

id::define!(pub MessageId<8, 12>);

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Message {
    pub author: UserId,
    pub timestamp: u64,
    pub target: MessageTarget,
    pub content: String,
    pub reactions: UnorderedMap<UserId, Reaction>,
    pub comments: UnorderedSet<MessageId>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
#[serde(crate = "calimero_sdk::serde")]
pub enum MessageTarget {
    Bounty(BountyId),
    Bid(BidId),
    Assignment(AssignmentId),
    // User(UserId), // todo! should we allow folks to comment on peoples profiles?
    Message(MessageId),
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Reaction {
    emoji: String,
    timestamp: u64,
}

#[derive(Debug, Error, Serialize)]
#[serde(crate = "calimero_sdk::serde")]
#[serde(tag = "kind", content = "data")]
pub enum MessageError {
    #[error("message target not found: {0:?}")]
    TargetNotFound(MessageTarget),
}

impl AppState {
    pub fn internal_post_message(
        &mut self,
        user_id: UserId,
        user: &mut User,
        target: MessageTarget,
        content: String,
    ) -> app::Result<MessageId> {
        let message_id = unique(|| MessageId::random(), |id| self.messages.contains(id))?;

        let timestamp = env::time_now();

        let message = Message {
            author: user_id,
            timestamp,
            target,
            content,
            reactions: UnorderedMap::new(),
            comments: UnorderedSet::new(),
        };

        let _ignored = user.messages.insert(message_id)?;

        let _ignored = self.messages.insert(message_id, message)?;

        Ok(message_id)
    }
}

#[app::logic]
impl AppState {
    pub fn post_message(
        &mut self,
        target: MessageTarget,
        content: String,
    ) -> app::Result<MessageId> {
        let user_id = self.current_user();

        let mut user = self.get_registered_user(&user_id)?;

        let parent = match target {
            MessageTarget::Bounty(id) => self.bounties.get(&id)?.map(|b| b.message),
            MessageTarget::Bid(id) => self.bids.get(&id)?.map(|b| b.message),
            MessageTarget::Assignment(id) => self.assignments.get(&id)?.map(|b| b.message),
            MessageTarget::Message(id) => Some(id),
        };

        let Some(parent) = parent else {
            app::bail!(MessageError::TargetNotFound(target));
        };

        let message_id = self.internal_post_message(
            user_id,
            &mut user,
            MessageTarget::Message(parent),
            content,
        )?;

        let _ignored = self.users.insert(user_id, user)?;

        Ok(message_id)
    }
}
