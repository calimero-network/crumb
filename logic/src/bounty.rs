use std::ops::Deref;

use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_sdk::{app, env};
use calimero_storage::collections::{UnorderedMap, UnorderedSet};

use crate::assignment::AssignmentId;
use crate::bid::BidId;
use crate::message::MessageId;
use crate::types::id::{Id, IdExt};
use crate::user::UserId;
use crate::utils::unique;
use crate::{AppState, LabelId};

#[derive(
    Eq, Copy, Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
#[borsh(crate = "calimero_sdk::borsh")]
#[serde(crate = "calimero_sdk::serde")]
pub struct BountyId(Id<8, 12>);

impl Deref for BountyId {
    type Target = Id<8, 12>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for BountyId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<[u8; 8]> for BountyId {
    fn from(id: [u8; 8]) -> Self {
        Self(Id::from(id))
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Bounty {
    pub is_epic: bool,
    pub author: UserId,
    pub description: String,
    pub reviewers: UnorderedSet<UserId>,
    pub labels: UnorderedSet<LabelId>,
    pub award: Option<u128>,
    pub bids: UnorderedMap<UserId, BidId>,
    pub assignments: UnorderedMap<UserId, AssignmentId>,
    pub status: BountyStatus,
    pub deadline: Option<u64>,
    pub parent: Option<BountyId>,
    pub children: UnorderedSet<BountyId>,
    pub comments: UnorderedSet<MessageId>,

    pub triaged_by: Option<UserId>,
    pub approved_by: Option<UserId>,
    pub closed_by: Option<UserId>,

    pub proposed_at: Option<u64>,
    pub triaged_at: Option<u64>,
    pub approved_at: Option<u64>,
    pub closed_at: Option<u64>,
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

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct CreateBountyRequest {
    pub is_epic: bool,
    pub description: String,
    pub reviewers: Vec<UserId>,
    pub labels: Vec<LabelId>,
    pub award: Option<u128>,
    pub deadline: Option<u64>,
    pub parent: Option<BountyId>,
}

#[app::logic]
impl AppState {
    pub fn create_bounty(&mut self, request: CreateBountyRequest) -> app::Result<BountyId> {
        self.ensure_registered()?;

        let author = UserId::from(env::executor_id());

        let bounty_id = unique(|| BountyId::random(), |id| self.bounties.contains(id))?;

        let bounty = Bounty {
            is_epic: request.is_epic,
            author,
            description: request.description,
            reviewers: request.reviewers.into_iter().collect(),
            labels: request.labels.into_iter().collect(),
            award: request.award,
            bids: UnorderedMap::new(),
            assignments: UnorderedMap::new(),
            status: BountyStatus::Proposed,
            deadline: request.deadline,
            parent: request.parent,
            children: UnorderedSet::new(),
            comments: UnorderedSet::new(),

            triaged_by: None,
            approved_by: None,
            closed_by: None,

            proposed_at: Some(env::time_now()),
            triaged_at: None,
            approved_at: None,
            closed_at: None,
        };

        let _ignored = self.bounties.insert(bounty_id, bounty);

        Ok(bounty_id)
    }
}
