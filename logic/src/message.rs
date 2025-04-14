use std::collections::BTreeMap;

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
use crate::utils::{borsh_char, unique};
use crate::AppState;

id::define!(pub MessageId<8, 12>);

const MAX_MESSAGE_LENGTH: usize = 1000;

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Message {
    pub author: UserId,
    pub timestamp: u64,
    pub target: MessageTarget,
    pub content: String,
    pub reactions: UnorderedSet<Reaction>,
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
    #[borsh(
        serialize_with = "borsh_char::ser",
        deserialize_with = "borsh_char::de"
    )]
    emoji: char,
    users: UnorderedMap<UserId, u64>,
}

#[derive(Debug, Error, Serialize)]
#[serde(crate = "calimero_sdk::serde")]
#[serde(tag = "kind", content = "data")]
pub enum Error {
    #[error("message not found: {0}")]
    MessageNotFound(MessageId),
    #[error("parent message not found: {0}")]
    ParentMessageNotFound(MessageId),
    #[error("message target not found: {0:?}")]
    MessageTargetNotFound(MessageTarget),
    #[error("message too long ({got} > {max})")]
    MessageTooLong { got: usize, max: usize },
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
            reactions: UnorderedSet::new(),
            comments: UnorderedSet::new(),
        };

        if let MessageTarget::Message(parent_id) = message.target {
            let Some(mut parent) = self.messages.get(&parent_id)? else {
                app::bail!(Error::ParentMessageNotFound(parent_id));
            };

            let _ignored = parent.comments.insert(message_id)?;

            let _ignored = self.messages.insert(parent_id, parent)?;
        }

        let _ignored = user.messages.insert(message_id)?;

        let _ignored = self.messages.insert(message_id, message)?;

        Ok(message_id)
    }

    pub fn internal_get_message(&self, message_id: MessageId) -> app::Result<Message> {
        let Some(message) = self.messages.get(&message_id)? else {
            app::bail!(Error::MessageNotFound(message_id));
        };

        Ok(message)
    }
}

fn validate_message(message: &str) -> app::Result<()> {
    if message.len() > MAX_MESSAGE_LENGTH {
        app::bail!(Error::MessageTooLong {
            got: message.len(),
            max: MAX_MESSAGE_LENGTH,
        });
    }
    Ok(())
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

        validate_message(&content)?;

        let parent = match target {
            MessageTarget::Bounty(id) => self.bounties.get(&id)?.map(|b| b.message),
            MessageTarget::Bid(id) => self.bids.get(&id)?.map(|b| b.message),
            MessageTarget::Assignment(id) => self.assignments.get(&id)?.map(|b| b.message),
            MessageTarget::Message(id) => Some(id),
        };

        let Some(parent) = parent else {
            app::bail!(Error::MessageTargetNotFound(target));
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct MessageView {
    pub id: MessageId,
    pub author: UserId,
    pub timestamp: u64,
    pub target: MessageTarget,
    pub content: String,
    pub reactions: Vec<ReactionView>,
    pub comments: Vec<MessageId>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct ReactionView {
    pub emoji: char,
    pub users: BTreeMap<UserId, u64>,
}

#[app::logic]
impl AppState {
    pub fn get_message(&self, message_id: MessageId) -> app::Result<MessageView> {
        let message = self.internal_get_message(message_id)?;

        let reactions = message
            .reactions
            .iter()?
            .map(|reaction| {
                let emoji = reaction.emoji;

                let users = reaction.users.entries()?.collect();

                Ok(ReactionView { emoji, users })
            })
            .collect::<app::Result<_>>()?;

        let comments = message.comments.iter()?.collect();

        Ok(MessageView {
            id: message_id,
            author: message.author,
            timestamp: message.timestamp,
            target: message.target,
            content: message.content,
            reactions,
            comments,
        })
    }
}
