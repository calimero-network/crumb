use std::collections::BTreeSet;
use std::sync::LazyLock;

use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_sdk::{app, env};
use calimero_storage::collections::UnorderedSet;
use thiserror::Error;

use crate::bid::BidId;
use crate::bounty::BountyId;
use crate::message::MessageId;
use crate::types::id;
use crate::AppState;

id::define!(pub UserId<32, 44>);

const MAX_USER_NAME_LENGTH: usize = 40;
const MAX_USER_SKILL_LENGTH: usize = 20;
const MAX_USER_LINK_LENGTH: usize = 200;

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

#[derive(Debug, Error, Serialize)]
#[serde(crate = "calimero_sdk::serde")]
#[serde(tag = "kind", content = "data")]
pub enum Error<'a> {
    #[error("user is not registered")]
    UserNotRegistered,
    #[error("user is already registered")]
    UserAlreadyRegistered,
    #[error("username cannot be empty")]
    UserNameCannotBeEmpty,
    #[error("username is too long ({max} > {got})")]
    UserNameTooLong { max: usize, got: usize },
    #[error("user skill is too long ({max} > {got}): {skill}")]
    UserSkillTooLong {
        skill: &'a str,
        max: usize,
        got: usize,
    },
    #[error("user link is too long ({max} > {got}): {link}")]
    UserLinkTooLong {
        link: &'a str,
        max: usize,
        got: usize,
    },
}

static EXECUTOR_ID: LazyLock<UserId> = std::sync::LazyLock::new(|| UserId::new(env::executor_id()));

impl AppState {
    pub fn current_user(&self) -> UserId {
        *EXECUTOR_ID
    }

    pub fn ensure_registered_user(&self, user_id: &UserId) -> app::Result<()> {
        if !self.users.contains(user_id)? {
            app::bail!(Error::UserNotRegistered);
        }

        Ok(())
    }

    pub fn get_registered_user(&self, user_id: &UserId) -> app::Result<User> {
        let Some(user) = self.users.get(user_id)? else {
            app::bail!(Error::UserNotRegistered);
        };

        Ok(user)
    }
}

fn validate_user_name(name: &str) -> app::Result<()> {
    if name.is_empty() {
        app::bail!(Error::UserNameCannotBeEmpty);
    }

    if name.len() > MAX_USER_NAME_LENGTH {
        app::bail!(Error::UserNameTooLong {
            max: MAX_USER_NAME_LENGTH,
            got: name.len(),
        });
    }

    Ok(())
}

fn validate_skill(skill: &str) -> app::Result<()> {
    if skill.len() > MAX_USER_SKILL_LENGTH {
        app::bail!(Error::UserSkillTooLong {
            skill,
            max: MAX_USER_SKILL_LENGTH,
            got: skill.len(),
        });
    }

    Ok(())
}

fn validate_link(link: &str) -> app::Result<()> {
    if link.len() > MAX_USER_LINK_LENGTH {
        app::bail!(Error::UserLinkTooLong {
            link,
            max: MAX_USER_LINK_LENGTH,
            got: link.len(),
        });
    }

    Ok(())
}

#[app::logic]
impl AppState {
    pub fn register(
        &mut self,
        name: Option<String>,
        skills: BTreeSet<String>,
        links: BTreeSet<String>,
    ) -> app::Result<UserId> {
        let user_id = self.current_user();

        if self.users.contains(&user_id)? {
            app::bail!(Error::UserAlreadyRegistered);
        }

        if let Some(name) = &name {
            validate_user_name(name)?;
        }

        let skills = skills
            .into_iter()
            .map(|s| validate_skill(&s).map(|_| s))
            .collect::<app::Result<_>>()?;

        let links = links
            .into_iter()
            .map(|s| validate_link(&s).map(|_| s))
            .collect::<app::Result<_>>()?;

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
    pub name: Option<DeltaOperation<String>>,
    #[serde(default)]
    pub skills: Vec<DeltaOperation<String>>,
    #[serde(default)]
    pub links: Vec<DeltaOperation<String>>,
}

#[derive(Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub enum DeltaOperation<T> {
    Add(T),
    Remove(Option<T>),
}

#[app::logic]
impl AppState {
    pub fn update_user(&mut self, user_id: UserId, delta: UserDelta) -> app::Result<()> {
        let mut user = self.get_registered_user(&user_id)?;

        if let Some(op) = delta.name {
            match op {
                DeltaOperation::Add(name) => {
                    validate_user_name(&name)?;

                    user.name = Some(name)
                }
                DeltaOperation::Remove(_) => user.name = None,
            }
        }
        for op in delta.skills {
            match op {
                DeltaOperation::Add(skill) => {
                    validate_skill(&skill)?;

                    let _ignored = user.skills.insert(skill)?;
                }
                DeltaOperation::Remove(skill) => {
                    if let Some(skill) = skill {
                        let _ignored = user.skills.remove(&skill)?;
                    } else {
                        user.skills.clear()?
                    }
                }
            };
        }

        for op in delta.links {
            match op {
                DeltaOperation::Add(link) => {
                    validate_link(&link)?;

                    let _ignored = user.links.insert(link)?;
                }
                DeltaOperation::Remove(link) => {
                    if let Some(link) = link {
                        let _ignored = user.links.remove(&link)?;
                    } else {
                        user.links.clear()?
                    }
                }
            };
        }

        let _ignored = self.users.insert(user_id, user)?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct UserViewBrief {
    pub id: UserId,
    pub name: Option<String>,
    // pub rank: Option<u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "calimero_sdk::serde")]
pub struct UserView {
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
    pub fn get_user_brief(&self, user_id: UserId) -> app::Result<Option<UserViewBrief>> {
        let user = self.users.get(&user_id)?;

        let Some(user) = user else {
            return Ok(None);
        };

        Ok(Some(UserViewBrief {
            id: user_id,
            name: user.name,
        }))
    }

    pub fn get_user(&self, user_id: UserId) -> app::Result<Option<UserView>> {
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

        Ok(Some(UserView {
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
