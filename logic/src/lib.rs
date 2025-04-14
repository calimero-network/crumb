use calimero_sdk::app;
use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_storage::collections::UnorderedMap;

mod assignment;
mod bid;
mod bounty;
mod event;
mod label;
mod message;
mod paging;
mod private;
mod types;
mod user;
mod utils;

use assignment::{Assignment, AssignmentId};
use bid::{Bid, BidId};
use bounty::{Bounty, BountyId};
use event::Event;
use label::{Label, LabelId};
use message::{Message, MessageId};
use user::{User, UserId};

#[app::state(emits = Event)]
#[derive(Default, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct AppState {
    users: UnorderedMap<UserId, User>,
    bids: UnorderedMap<BidId, Bid>,
    assignments: UnorderedMap<AssignmentId, Assignment>,
    bounties: UnorderedMap<BountyId, Bounty>,
    messages: UnorderedMap<MessageId, Message>,
    labels: UnorderedMap<LabelId, Label>,
}

#[app::logic]
impl AppState {
    #[app::init]
    pub fn init() -> AppState {
        AppState::default()
    }
}
