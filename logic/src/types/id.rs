use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use core::str::{self, FromStr};
use std::borrow::Cow;

use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::env;
use calimero_sdk::serde::{self, de, Deserialize, Serialize};

enum Dud<const N: usize> {}

#[derive(Eq, Ord, Copy, Clone, Debug, PartialEq, PartialOrd, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Id<const N: usize, const S: usize = 0> {
    bytes: [u8; N],
    _priv: PhantomData<Dud<S>>,
}

impl<const N: usize, const S: usize> Id<N, S> {
    const SIZE_ASSOC_GUARD: () = {
        let expected_size = (N + 1) * 4 / 3;
        let _guard = S - expected_size;
    };

    pub const fn new(id: [u8; N]) -> Self {
        let _guard = Self::SIZE_ASSOC_GUARD;

        Self {
            bytes: id,
            _priv: PhantomData,
        }
    }

    pub fn as_str<'a>(&self, buf: &'a mut [u8; S]) -> &'a str {
        let len = bs58::encode(&self.bytes)
            .onto(&mut buf[..])
            .expect("buffer too small");

        str::from_utf8(&buf[..len]).unwrap()
    }
}

impl<const N: usize, const S: usize> FromStr for Id<N, S> {
    type Err = bs58::decode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = [0; N];

        let _len = bs58::decode(s).onto(&mut buf[..])?;

        Ok(Self::new(buf))
    }
}

impl<const N: usize, const S: usize> AsRef<[u8]> for Id<N, S> {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl<const N: usize, const S: usize> AsRef<[u8; N]> for Id<N, S> {
    fn as_ref(&self) -> &[u8; N] {
        &self.bytes
    }
}

impl<const N: usize, const S: usize> Deref for Id<N, S> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl<const N: usize, const S: usize> From<[u8; N]> for Id<N, S> {
    fn from(id: [u8; N]) -> Self {
        Self::new(id)
    }
}

impl<const N: usize, const S: usize> fmt::Display for Id<N, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(self.as_str(&mut [0; S]))
    }
}

impl<const N: usize, const S: usize> Serialize for Id<N, S> {
    fn serialize<O>(&self, serializer: O) -> Result<O::Ok, O::Error>
    where
        O: serde::Serializer,
    {
        let mut buf = [0; S];

        serializer.serialize_str(self.as_str(&mut buf))
    }
}

impl<'de, const N: usize, const S: usize> Deserialize<'de> for Id<N, S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(crate = "calimero_sdk::serde")]
        struct Container<'a>(#[serde(borrow)] Cow<'a, str>);

        let encoded = Container::deserialize(deserializer)?;

        Self::from_str(&*encoded.0).map_err(de::Error::custom)
    }
}

pub trait IdExt<const N: usize> {
    fn random() -> Self;
}

impl<const N: usize, T> IdExt<N> for T
where
    T: From<[u8; N]>,
{
    fn random() -> Self {
        let mut bytes = [0; N];

        env::random_bytes(&mut bytes);

        Self::from(bytes)
    }
}
