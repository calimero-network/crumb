use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_storage::collections::UnorderedMap;

use crate::bid::BidId;
use crate::bounty::BountyId;
use crate::types::id;
use crate::user::UserId;

id::define!(pub MessageId<8, 12>);

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Message {
    pub author: UserId,
    pub timestamp: u64,
    pub target: MessageTarget,
    pub content: String,
    pub reactions: UnorderedMap<UserId, Reaction>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum MessageTarget {
    Bounty(BountyId),
    Bid(BidId),
    Assignment(BidId),
    User(UserId),
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Reaction {
    emoji: String,
    timestamp: u64,
}
