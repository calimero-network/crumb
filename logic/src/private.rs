use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use calimero_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use calimero_sdk::{app, env};

use crate::paging::PagingSessions;
use crate::types::id;
use crate::utils::IntoResult;

id::define!(pub StateKey<13, 18>);

const PRIVATE_STATE_KEY: StateKey = StateKey::new(*b"PRIVATE_STATE");

#[derive(Default, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct PrivateRootState {
    paging_sessions: PagingSessions,
}

#[derive(BorshSerialize, BorshDeserialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct EntryHandle<T> {
    key: StateKey,
    dud: PhantomData<T>,
}

impl<T> EntryHandle<T> {
    pub fn new(key: StateKey) -> Self {
        Self {
            key,
            dud: PhantomData,
        }
    }

    pub fn root() -> EntryHandle<PrivateRootState> {
        EntryHandle {
            key: PRIVATE_STATE_KEY,
            dud: PhantomData,
        }
    }
}

impl<T: BorshDeserialize> EntryHandle<T> {
    pub fn get(&self) -> app::Result<Option<EntryRef<T>>> {
        let key = self.key;

        let Some(data) = env::storage_read(key.as_ref()) else {
            return Ok(None);
        };

        let state = T::try_from_slice(&data)?;

        Ok(Some(EntryRef { key, state }))
    }

    pub fn get_or_init_with<R: IntoResult<T>>(
        &self,
        f: impl FnOnce() -> R,
    ) -> app::Result<EntryRef<T>>
    where
        R::Error: core::error::Error,
    {
        let key = self.key;

        let Some(data) = env::storage_read(key.as_ref()) else {
            let state = f().into_result()?;

            return Ok(EntryRef { key, state });
        };

        let state = T::try_from_slice(&data)?;

        Ok(EntryRef { key, state })
    }

    pub fn get_or_default(&self) -> app::Result<EntryRef<T>>
    where
        T: Default,
    {
        self.get_or_init_with(T::default)
    }
}

pub struct EntryRef<T> {
    key: StateKey,
    state: T,
}

impl<T> EntryRef<T> {
    pub fn as_mut(&mut self) -> EntryMut<'_, T>
    where
        T: BorshSerialize,
    {
        EntryMut {
            key: self.key,
            state: &mut self.state,
        }
    }
}

impl<T> Deref for EntryRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

pub struct EntryMut<'a, T: BorshSerialize> {
    key: StateKey,
    state: &'a mut T,
}

impl<T: BorshSerialize> Deref for EntryMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<T: BorshSerialize> DerefMut for EntryMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<T: BorshSerialize> Drop for EntryMut<'_, T> {
    fn drop(&mut self) {
        let data = borsh::to_vec(self.state).unwrap();

        let _ignored = env::storage_write(PRIVATE_STATE_KEY.as_ref(), &data);
    }
}
