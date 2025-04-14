use std::collections::BTreeSet;

use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_sdk::{app, env};
use calimero_storage::collections::UnorderedSet;
use thiserror::Error;

use crate::assignment::AssignmentId;
use crate::bid::BidId;
use crate::message::{MessageId, MessageTarget};
use crate::paging::ResumptionToken;
use crate::types::id::{self, IdExt};
use crate::user::UserId;
use crate::utils::{truncate_string, unique};
use crate::{AppState, LabelId};

id::define!(pub BountyId<8, 12>);

const MAX_BOUNTY_TITLE_LENGTH: usize = 80;
const MAX_BOUNTY_DESCRIPTION_LENGTH: usize = 10_000;
const MAX_NUMBER_OF_REQUIRED_REVIEWS: usize = 20;
const MAX_NUMBER_OF_LABELS: usize = 20;

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Bounty {
    pub title: String,
    pub author: UserId,
    pub message: MessageId,

    pub award: Option<u128>,
    pub status: BountyStatus,
    pub is_epic: bool,
    pub deadline: Option<u64>,

    pub labels: UnorderedSet<LabelId>,
    pub reviewers: UnorderedSet<UserId>,

    pub bids: UnorderedSet<BidId>,
    pub assignments: UnorderedSet<AssignmentId>,

    pub parent: Option<BountyId>,
    pub children: UnorderedSet<BountyId>,

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

#[derive(Debug, Error, Serialize)]
#[serde(crate = "calimero_sdk::serde")]
#[serde(tag = "kind", content = "data")]
pub enum Error {
    #[error("bounty not found")]
    BountyNotFound,
    #[error("bounty title too long ({got} > {max})")]
    BountyTitleTooLong { got: usize, max: usize },
    #[error("bounty description too long ({got} > {max})")]
    BountyDescriptionTooLong { got: usize, max: usize },
    #[error("bounty reviewers limit exceeded (max {max})")]
    BountyReviewersLimitExceeded { max: usize },
    #[error("bounty labels limit exceeded (max {max})")]
    BountyLabelsLimitExceeded { max: usize },
}

fn validate_bounty_title(title: &str) -> app::Result<()> {
    if title.len() > MAX_BOUNTY_TITLE_LENGTH {
        app::bail!(Error::BountyTitleTooLong {
            got: title.len(),
            max: MAX_BOUNTY_TITLE_LENGTH,
        });
    }

    Ok(())
}

fn validate_bounty_description(description: &str) -> app::Result<()> {
    if description.len() > MAX_BOUNTY_DESCRIPTION_LENGTH {
        app::bail!(Error::BountyDescriptionTooLong {
            got: description.len(),
            max: MAX_BOUNTY_DESCRIPTION_LENGTH,
        });
    }

    Ok(())
}

fn validate_increment_bounty_reviewers(reviewers_count: usize) -> app::Result<()> {
    if reviewers_count > MAX_NUMBER_OF_REQUIRED_REVIEWS {
        app::bail!(Error::BountyReviewersLimitExceeded {
            max: MAX_NUMBER_OF_REQUIRED_REVIEWS,
        });
    }

    Ok(())
}

fn validate_bounty_labels(labels_count: usize) -> app::Result<()> {
    if labels_count > MAX_NUMBER_OF_LABELS {
        app::bail!(Error::BountyLabelsLimitExceeded {
            max: MAX_NUMBER_OF_LABELS,
        });
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct CreateBountyRequest {
    pub is_epic: bool,
    pub title: String,
    pub description: String,
    pub reviewers: BTreeSet<UserId>,
    pub labels: BTreeSet<LabelId>,
    pub award: Option<u128>,
    pub deadline: Option<u64>,
    pub parent: Option<BountyId>,
}

#[app::logic]
impl AppState {
    pub fn create_bounty(&mut self, request: CreateBountyRequest) -> app::Result<BountyId> {
        let user_id = self.current_user();

        let mut user = self.get_registered_user(&user_id)?;

        validate_bounty_title(&request.title)?;
        validate_bounty_description(&request.description)?;
        validate_increment_bounty_reviewers(request.reviewers.len())?;
        validate_bounty_labels(request.labels.len())?;

        let bounty_id = unique(|| BountyId::random(), |id| self.bounties.contains(id))?;

        let message_id = self.internal_post_message(
            user_id,
            &mut user,
            MessageTarget::Bounty(bounty_id),
            request.description,
        )?;

        let now = env::time_now();

        let bounty = Bounty {
            title: request.title,
            author: user_id,
            message: message_id,

            award: request.award,
            status: BountyStatus::Proposed,
            is_epic: request.is_epic,
            deadline: request.deadline,

            labels: request.labels.into_iter().collect(),
            reviewers: request.reviewers.into_iter().collect(),

            bids: UnorderedSet::new(),
            assignments: UnorderedSet::new(),

            parent: request.parent,
            children: UnorderedSet::new(),

            triaged_by: None,
            approved_by: None,
            closed_by: None,

            proposed_at: Some(now),
            triaged_at: None,
            approved_at: None,
            closed_at: None,
            updated_at: Some(now),
        };

        let _ignored = user.bounties.insert(bounty_id)?;

        let _ignored = self.users.insert(user_id, user)?;
        let _ignored = self.bounties.insert(bounty_id, bounty)?;

        Ok(bounty_id)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub enum BountyStatusLite {
    Proposed,
    Triaged,
    Approved,
    Expired,
    Abandoned,
    Completed,
}

impl From<&BountyStatus> for BountyStatusLite {
    fn from(status: &BountyStatus) -> Self {
        match status {
            BountyStatus::Proposed => BountyStatusLite::Proposed,
            BountyStatus::Triaged => BountyStatusLite::Triaged,
            BountyStatus::Approved => BountyStatusLite::Approved,
            BountyStatus::Closed { reason } => match reason {
                ClosureReason::Expired => BountyStatusLite::Expired,
                ClosureReason::Abandoned => BountyStatusLite::Abandoned,
                ClosureReason::Completed { .. } => BountyStatusLite::Completed,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct BountyViewBrief {
    id: BountyId,
    title: String,
    author: UserId,
    description: String,
    status: BountyStatusLite,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct BountyView {
    id: BountyId,
    title: String,
    author: UserId,
    message: MessageId, // <- fetch description via message API instead

    award: Option<u128>,
    status: BountyStatusLite,
    is_epic: bool,
    deadline: Option<u64>,

    labels: Vec<LabelId>,
    reviewers: Vec<UserId>,

    bids: Vec<BidId>,
    assignments: Vec<AssignmentId>,

    parent: Option<BountyId>,
    children: Vec<BountyId>,

    triaged_by: Option<UserId>,
    approved_by: Option<UserId>,
    closed_by: Option<UserId>,

    proposed_at: Option<u64>,
    triaged_at: Option<u64>,
    approved_at: Option<u64>,
    closed_at: Option<u64>,
    updated_at: Option<u64>,
}

#[app::logic]
impl AppState {
    pub fn get_bounty_brief(&self, bounty_id: BountyId) -> app::Result<BountyViewBrief> {
        let Some(bounty) = self.bounties.get(&bounty_id)? else {
            app::bail!(Error::BountyNotFound);
        };

        let message = self.internal_get_message(&bounty.message)?;

        let status = BountyStatusLite::from(&bounty.status);

        Ok(BountyViewBrief {
            id: bounty_id,
            title: bounty.title,
            author: bounty.author,
            description: truncate_string(&message.content, 100).into(),
            status,
        })
    }

    pub fn get_bounty(&self, bounty_id: BountyId) -> app::Result<BountyView> {
        let Some(bounty) = self.bounties.get(&bounty_id)? else {
            app::bail!(Error::BountyNotFound);
        };

        let status = BountyStatusLite::from(&bounty.status);

        let labels = bounty.labels.iter()?.collect();
        let reviewers = bounty.reviewers.iter()?.collect();
        let bids = bounty.bids.iter()?.collect();
        let assignments = bounty.assignments.iter()?.collect();
        let children = bounty.children.iter()?.collect();

        Ok(BountyView {
            id: bounty_id,
            title: bounty.title,
            author: bounty.author,
            message: bounty.message,

            award: bounty.award,
            status,
            is_epic: bounty.is_epic,
            deadline: bounty.deadline,

            labels,
            reviewers,

            bids,
            assignments,

            parent: bounty.parent,
            children,

            triaged_by: bounty.triaged_by,
            approved_by: bounty.approved_by,
            closed_by: bounty.closed_by,

            proposed_at: bounty.proposed_at,
            triaged_at: bounty.triaged_at,
            approved_at: bounty.approved_at,
            closed_at: bounty.closed_at,
            updated_at: bounty.updated_at,
        })
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
