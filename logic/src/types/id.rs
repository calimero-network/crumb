use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use core::str::{self, FromStr};
use std::borrow::Cow;

use calimero_sdk::borsh::{BorshDeserialize, BorshSerialize};
use calimero_sdk::env;
use calimero_sdk::serde::{self, de, Deserialize, Serialize};

enum Dud<const N: usize> {}

#[derive(Eq, Copy, Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "calimero_sdk::borsh")]
pub struct Id<const N: usize, const M: usize> {
    bytes: [u8; N],
    _priv: PhantomData<Dud<M>>,
}

impl<const N: usize, const M: usize> Id<N, M> {
    const EXPECTED_STR_LEN: usize = (N + 1) * 4 / 3;

    pub fn new(id: [u8; N]) -> Self {
        debug_assert!(
            M == Self::EXPECTED_STR_LEN,
            "Id<{N}, {M}> is invalid expected Id<{N}, {}>",
            Self::EXPECTED_STR_LEN
        );

        Self {
            bytes: id,
            _priv: PhantomData,
        }
    }

    pub fn as_str<'a>(&self, buf: &'a mut [u8; M]) -> &'a str {
        let len = bs58::encode(&self.bytes)
            .onto(&mut buf[..])
            .expect("buffer too small");

        str::from_utf8(&buf[..len]).unwrap()
    }
}

impl<const N: usize, const M: usize> FromStr for Id<N, M> {
    type Err = bs58::decode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = [0; N];

        let _len = bs58::decode(s).onto(&mut buf[..])?;

        Ok(Self::new(buf))
    }
}

impl<const N: usize, const M: usize> AsRef<[u8]> for Id<N, M> {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl<const N: usize, const M: usize> AsRef<[u8; N]> for Id<N, M> {
    fn as_ref(&self) -> &[u8; N] {
        &self.bytes
    }
}

impl<const N: usize, const M: usize> Deref for Id<N, M> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl<const N: usize, const M: usize> From<[u8; N]> for Id<N, M> {
    fn from(id: [u8; N]) -> Self {
        Self::new(id)
    }
}

impl<const N: usize, const M: usize> fmt::Display for Id<N, M> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(self.as_str(&mut [0; M]))
    }
}

impl<const N: usize, const M: usize> Serialize for Id<N, M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = [0; M];

        serializer.serialize_str(self.as_str(&mut buf))
    }
}

impl<'de, const N: usize, const M: usize> Deserialize<'de> for Id<N, M> {
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
