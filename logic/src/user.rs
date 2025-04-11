use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::serde::{Deserialize, Serialize};
use calimero_sdk::{app, env};
use calimero_storage::collections::Vector;

use crate::bid::BidId;
use crate::bounty::BountyId;
use crate::message::MessageId;
use crate::types::id::Id;
use crate::AppState;

#[derive(Eq, Debug, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
#[serde(crate = "calimero_sdk::serde")]
pub struct UserId(Id<32, 44>);

impl From<[u8; 32]> for UserId {
    fn from(id: [u8; 32]) -> Self {
        Self(Id::from(id))
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct User {
    pub name: Option<String>,
    pub skills: Vector<String>,
    pub links: Vector<String>,
    pub bids: Vector<BidId>,
    pub total_reward: u128,
    pub assignments: Vector<BidId>,
    pub bounties: Vector<BountyId>,
    pub messages: Vector<MessageId>,
    pub remarks: Vector<UserRemarks>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct UserRemarks {
    pub review: f32, // 0.0 - 5.0
    pub message: MessageId,
}

#[app::logic]
impl AppState {
    pub fn self_is_registered(&self) -> Result<bool, ()> {
        todo!();
        // let user = env::executor_id().into();

        // self.users.contains(&user).map_err(Into::into)
    }

    // pub fn get_self(&self) -> User {
    //     self.get_user_id()
    // }
}
