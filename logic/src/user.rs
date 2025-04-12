use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_sdk::{app, env};
use calimero_storage::collections::UnorderedSet;

use crate::bid::BidId;
use crate::bounty::BountyId;
use crate::error::Error;
use crate::message::MessageId;
use crate::types::id;
use crate::AppState;

id::define!(pub UserId<32>);

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct User {
    pub name: Option<String>,
    pub skills: UnorderedSet<String>,
    pub links: UnorderedSet<String>,
    pub total_reward: u128,
    pub bids: UnorderedSet<BidId>,
    pub assignments: UnorderedSet<BidId>,
    pub bounties: UnorderedSet<BountyId>,
    pub messages: UnorderedSet<MessageId>,
    pub remarks: UnorderedSet<UserRemarks>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct UserRemarks {
    pub review: f32, // 0.0 - 5.0
    pub message: MessageId,
}

macro_rules! ensure_registered {
    ($user:expr) => {{
        let Some(user): Option<User> = $user else {
            app::bail!(Error::NotRegistered);
        };

        user
    }};
}

#[app::logic]
impl AppState {
    pub fn ensure_registered(&self) -> app::Result<()> {
        let user = UserId::from(env::executor_id());

        if !self.users.contains(&user)? {
            app::bail!(Error::NotRegistered);
        }

        Ok(())
    }

    pub fn register(
        &mut self,
        name: Option<String>,
        skills: Vec<String>,
        links: Vec<String>,
    ) -> app::Result<UserId> {
        let user_id = UserId::from(env::executor_id());

        if self.users.contains(&user_id)? {
            app::bail!(Error::AlreadyRegistered);
        }

        let skills = skills.into_iter().collect();
        let links = links.into_iter().collect();

        let user = User {
            name,
            skills,
            links,
            total_reward: 0,
            bids: UnorderedSet::new(),
            assignments: UnorderedSet::new(),
            bounties: UnorderedSet::new(),
            messages: UnorderedSet::new(),
            remarks: UnorderedSet::new(),
        };

        let _ignored = self.users.insert(user_id, user)?;

        Ok(user_id)
    }
}

#[derive(Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct UserDelta {
    #[serde(default)]
    pub name: Option<(DeltaOperation, String)>,
    #[serde(default)]
    pub skills: Vec<(DeltaOperation, String)>,
    #[serde(default)]
    pub links: Vec<(DeltaOperation, String)>,
}

#[derive(Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub enum DeltaOperation {
    Add,
    Remove,
}

#[app::logic]
impl AppState {
    pub fn update_user(&mut self, user_id: UserId, delta: UserDelta) -> app::Result<()> {
        let mut user = ensure_registered!(self.users.get(&user_id)?);

        if let Some((op, name)) = delta.name {
            match op {
                DeltaOperation::Add => user.name = Some(name),
                DeltaOperation::Remove => user.name = None,
            }
        }
        for (op, skill) in delta.skills {
            let _ignored = match op {
                DeltaOperation::Add => user.skills.insert(skill)?,
                DeltaOperation::Remove => user.skills.remove(&skill)?,
            };
        }

        for (op, link) in delta.links {
            let _ignored = match op {
                DeltaOperation::Add => user.links.insert(link)?,
                DeltaOperation::Remove => user.links.remove(&link)?,
            };
        }

        self.users.insert(user_id, user)?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct UserBrief {
    pub id: UserId,
    pub name: Option<String>,
    // pub rank: Option<u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct UserLite {
    pub id: UserId,
    pub name: Option<String>,
    pub skills: Vec<String>,
    pub links: Vec<String>,
    pub total_reward: u128,
    pub bids: Vec<BidId>,
    pub assignments: Vec<BidId>,
    pub bounties: Vec<BountyId>,
    pub messages: Vec<MessageId>,
}

#[app::logic]
impl AppState {
    pub fn get_user_brief(&self, user_id: UserId) -> app::Result<Option<UserBrief>> {
        let user = self.users.get(&user_id)?;

        let Some(user) = user else {
            return Ok(None);
        };

        Ok(Some(UserBrief {
            id: user_id,
            name: user.name,
        }))
    }

    pub fn get_user_lite(&self, user_id: UserId) -> app::Result<Option<UserLite>> {
        let user = self.users.get(&user_id)?;

        let Some(user) = user else {
            return Ok(None);
        };

        let links = user.links.iter()?;
        let skills = user.skills.iter()?;
        let bids = user.bids.iter()?;
        let assignments = user.assignments.iter()?;
        let bounties = user.bounties.iter()?;
        let messages = user.messages.iter()?;

        Ok(Some(UserLite {
            id: user_id,
            name: user.name,
            skills: skills.take(3).collect(),
            links: links.take(3).collect(),
            total_reward: user.total_reward,
            bids: bids.take(3).collect(),
            assignments: assignments.take(3).collect(),
            bounties: bounties.take(3).collect(),
            messages: messages.take(3).collect(),
        }))
    }
}

#[app::logic]
impl AppState {
    // pub fn submit_remark(
    //     &mut self,
    //     user_id: UserId,
    //     review: f32,
    //     message_id: MessageId,
    // ) -> app::Result<()> {
    //     let mut user = ensure_registered!(self.users.get(&user_id)?);

    //     let remark = UserRemarks {
    //         review,
    //         message: message_id,
    //     };

    //     let _ignored = user.remarks.insert(remark)?;

    //     self.users.insert(user_id, user)?;

    //     Ok(())
    // }
}
