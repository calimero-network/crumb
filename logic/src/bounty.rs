use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_sdk::{app, env};
use calimero_storage::collections::{UnorderedMap, UnorderedSet};

use crate::assignment::AssignmentId;
use crate::bid::BidId;
use crate::message::MessageId;
use crate::paging::ResumptionToken;
use crate::types::id::{self, IdExt};
use crate::user::UserId;
use crate::utils::unique;
use crate::{AppState, LabelId};

id::define!(pub BountyId<8, 12>);

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
    pub updated_at: Option<u64>,
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
        let author = self.current_user();

        self.ensure_registered_user(&author)?;

        let bounty_id = unique(|| BountyId::random(), |id| self.bounties.contains(id))?;

        let now = env::time_now();

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

            proposed_at: Some(now),
            triaged_at: None,
            approved_at: None,
            closed_at: None,
            updated_at: Some(now),
        };

        let _ignored = self.bounties.insert(bounty_id, bounty);

        Ok(bounty_id)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct BountyFilter {
    author: Option<UserId>,
    status: Option<BountyStatusFilter>,
    labels: Option<Vec<LabelId>>,
    reviewers: Option<Vec<UserId>>,
    deadline: Option<u64>,
    parent: Option<BountyId>,
    bid_by: Option<UserId>,
    assigned_to: Option<UserId>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub enum BountyStatusFilter {
    Proposed,
    Triaged,
    Approved,
    Expired,
    Abandoned,
    Completed,
}

impl BountyStatusFilter {
    fn against(&self, status: &BountyStatus) -> bool {
        match (self, status) {
            (BountyStatusFilter::Proposed, BountyStatus::Proposed)
            | (BountyStatusFilter::Triaged, BountyStatus::Triaged)
            | (BountyStatusFilter::Approved, BountyStatus::Approved) => true,
            (status, BountyStatus::Closed { reason }) => match (status, reason) {
                (BountyStatusFilter::Expired, ClosureReason::Expired)
                | (BountyStatusFilter::Abandoned, ClosureReason::Abandoned)
                | (BountyStatusFilter::Completed, ClosureReason::Completed { .. }) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct BountySortBy {
    field: SortField,
    order: SortOrder,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub enum SortField {
    ProposedAt,
    TriagedAt,
    ApprovedAt,
    ClosedAt,
    UpdatedAt,
    Deadline,
    Award,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
enum SortOrder {
    Ascending,
    Descending,
}

#[app::logic]
impl AppState {
    pub fn list_bounties(
        &self,
        resume: Option<ResumptionToken>,
        filter: Option<BountyFilter>,
        sortby: Option<BountySortBy>,
        offset: Option<usize>,
        length: Option<usize>,
    ) -> app::Result<(Vec<BountyId>, Option<ResumptionToken>)> {
        let user_id = self.current_user();

        self.ensure_registered_user(&user_id)?;

        // todo! integrate paginated resumption
        let _resume = resume;

        // vv~~ condition if `resume` is defined

        let bounties = self.bounties.entries()?;

        let filtered = bounties
            .map(|(id, bounty)| {
                if let Some(filter) = &filter {
                    if let Some(author) = &filter.author {
                        if bounty.author != *author {
                            return Ok(None);
                        }
                    }

                    if let Some(status) = &filter.status {
                        if !status.against(&bounty.status) {
                            return Ok(None);
                        }
                    }

                    // the others
                }

                Ok(Some((id, bounty)))
            })
            .filter_map(|e| e.transpose());

        let mut bounties = filtered.collect::<app::Result<Vec<_>>>()?;

        if let Some(sortby) = sortby {
            bounties.sort_by(|(_, a), (_, b)| {
                let ord = match sortby.field {
                    SortField::ProposedAt => a.proposed_at.cmp(&b.proposed_at),
                    SortField::TriagedAt => a.triaged_at.cmp(&b.triaged_at),
                    SortField::ApprovedAt => a.approved_at.cmp(&b.approved_at),
                    SortField::ClosedAt => a.closed_at.cmp(&b.closed_at),
                    SortField::UpdatedAt => a.updated_at.cmp(&b.updated_at),
                    SortField::Deadline => a.deadline.cmp(&b.deadline),
                    SortField::Award => a.award.cmp(&b.award),
                };

                match sortby.order {
                    SortOrder::Ascending => ord,
                    SortOrder::Descending => ord.reverse(),
                }
            });
        }

        // now we can save `bounties`, and get a `ResumptionToken`

        // ^^~~ condition if `resume` is defined

        let offset = offset.unwrap_or_default();
        let length = length.unwrap_or(bounties.len());

        let bounties = bounties.into_iter().map(|(id, _)| id);

        let bounties = bounties.skip(offset).take(length).collect();

        Ok((bounties, resume))
    }
}
