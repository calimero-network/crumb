use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::{app, env};
use calimero_storage::collections::{UnorderedMap, Vector};

#[app::event]
pub enum Event {
    Increased(u32),
    Reset,
}

type BidId = [u8; 8];
type UserId = [u8; 32];
type LabelId = [u8; 8];
type BountyId = [u8; 8];
type MessageId = [u8; 8];
type AssignmentId = [u8; 8];

#[app::state(emits = Event)]
#[derive(Default, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct AppState {
    users: UnorderedMap<UserId, User>,
    bids: UnorderedMap<BidId, Bid>,
    assignments: UnorderedMap<AssignmentId, Assignment>,
    bounties: UnorderedMap<BountyId, Bounty>,
    messages: UnorderedMap<MessageId, Message>,
    labels: UnorderedMap<LabelId, String>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct User {
    name: Option<String>,
    skills: Vector<String>,
    links: Vector<String>,
    bids: Vector<BidId>,
    total_reward: u128,
    assignments: Vector<BidId>,
    bounties: Vector<BountyId>,
    messages: Vector<MessageId>,
    remarks: Vector<UserRemarks>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct UserRemarks {
    review: f32, // 0.0 - 5.0
    message: MessageId,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Bounty {
    is_epic: bool,
    author: String,
    description: String,
    reviewers: Vector<UserId>,
    labels: Vector<LabelId>,
    award: Option<u128>,
    bids: UnorderedMap<UserId, BidId>,
    assignment: UnorderedMap<UserId, AssignmentId>,
    status: BountyStatus,
    deadline: Option<u64>,
    parent: Option<BountyId>,
    children: Vector<BountyId>,
    comments: Vector<MessageId>,

    triaged_by: Option<UserId>,
    approved_by: Option<UserId>,
    closed_by: Option<UserId>,

    proposed_at: Option<u64>,
    triaged_at: Option<u64>,
    approved_at: Option<u64>,
    closed_at: Option<u64>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum BountyStatus {
    Proposed,
    Triaged,
    Approved,
    Closed { reason: ClosureReason },
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum ClosureReason {
    Completed { assignment: AssignmentId },
    Abandoned,
    Expired,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Bid {
    assignment: Option<AssignmentId>,
    brief: String,
    bounty: BountyId,
    status: BidStatus,
    expiry: Option<u64>,
    reward: Option<Reward>,
    duration: Option<u64>,
    comments: Vector<MessageId>,

    proposed_at: Option<u64>,
    approved_at: Option<u64>,
    retracted_at: Option<u64>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Reward {
    amount: u128,
    account: String,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum BidStatus {
    Proposed,
    Approved,
    Retracted { reason: Option<String> },
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Assignment {
    bid: Option<BidId>,
    bounty: BountyId,
    links: Vector<String>,
    status: AssignmentStatus,
    expiry: Option<u64>,
    comments: Vector<MessageId>,

    received_at: Option<u64>,
    in_progress_at: Option<u64>,
    completed_at: Option<u64>,
    abandoned_at: Option<u64>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub enum AssignmentStatus {
    Received,
    InProgress,
    Completed,
    Abandoned { reason: Option<String> },
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Message {
    author: UserId,
    timestamp: u64,
    target: MessageTarget,
    content: String,
    reactions: UnorderedMap<UserId, Reaction>,
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

#[app::logic]
impl AppState {
    #[app::init]
    pub fn init() -> AppState {
        AppState::default()
    }
}
