use std::collections::BTreeMap;

use calimero_sdk::app;
use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};

use crate::private::{EntryHandle, EntryRef, StateKey};
use crate::types::id::{self, IdExt};
use crate::utils::unique;

id::define!(pub ResumptionToken<13, 18>);

#[derive(Default, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct PagingSessions {
    sessions: BTreeMap<ResumptionToken, EntryHandle<PagingSession>>,
}

#[derive(BorshSerialize, BorshDeserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct PagingSession {
    items: EntryHandle<SessionItems>,
}

#[derive(Default, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct SessionItems {
    // figure out some efficient way to get pages of items
}

impl PagingSessions {
    pub fn get(&self, token: ResumptionToken) -> app::Result<EntryRef<PagingSession>> {
        if let Some(handle) = self.sessions.get(&token) {
            if let Some(session) = handle.get()? {
                return Ok(session);
            }
        }

        app::bail!("invalid resumption token")
    }

    pub fn new_session(&mut self) -> app::Result<EntryRef<PagingSession>> {
        let token = unique(
            || ResumptionToken::random(),
            |t| self.sessions.contains_key(t),
        )?;

        let handle = EntryHandle::new(*token.transmute_ref());

        let handle = self.sessions.entry(token).or_insert(handle);

        handle.get_or_init_with(|| PagingSession {
            items: EntryHandle::new(StateKey::random()),
        })
    }
}
