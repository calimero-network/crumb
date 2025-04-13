use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use core::str::{self, FromStr};
use std::borrow::Cow;
use std::mem;

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
    #[doc(hidden)]
    pub const SIZE_GUARD: () = {
        let expected_size = (N + 1) * 4 / 3;
        let _guard = S - expected_size;
    };

    pub const fn new(id: [u8; N]) -> Self {
        let _guard = Self::SIZE_GUARD;

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

pub trait IdExt<const N: usize, const M: usize>: Sized {
    fn random() -> Self
    where
        Self: From<[u8; N]>;

    fn transmute_ref<T>(&self) -> &T
    where
        T: IdTransmute<N>,
        Self: IdTransmute<N>;

    fn transmute_slice<T>(items: &[Self]) -> &[T]
    where
        T: IdTransmute<N>,
        Self: IdTransmute<N>;
}

impl<const N: usize, const M: usize, T> IdExt<N, M> for T
where
    T: Deref<Target = Id<N, M>>,
{
    fn random() -> Self
    where
        T: From<[u8; N]>,
    {
        let mut bytes = [0; N];

        env::random_bytes(&mut bytes);

        Self::from(bytes)
    }

    fn transmute_ref<O>(&self) -> &O
    where
        O: IdTransmute<N>,
        Self: IdTransmute<N>,
    {
        unsafe { mem::transmute(self) }
    }

    fn transmute_slice<O>(items: &[Self]) -> &[O]
    where
        O: IdTransmute<N>,
        Self: IdTransmute<N>,
    {
        unsafe { mem::transmute(items) }
    }
}

#[doc(hidden)]
pub unsafe trait IdTransmute<const N: usize> {}

macro_rules! define {
    ($name:ident < $len:literal $(, $str:literal )? >) => {
        $crate::types::id::define!(@ () $name < $len $(, $str )? >);
    };
    (pub $name:ident < $len:literal $(, $str:literal )?>) => {
        $crate::types::id::define!(@ (pub) $name < $len $(, $str )? >);
    };
    (@ ( $($vis:tt)* ) $name:ident < $len:literal $(, $str:literal )? >) => {
        #[derive(
            ::core::cmp::Eq,
            ::core::cmp::Ord,
            ::core::marker::Copy,
            ::core::clone::Clone,
            ::core::fmt::Debug,
            ::core::cmp::PartialEq,
            ::core::cmp::PartialOrd,
            ::calimero_sdk::serde::Serialize,
            ::calimero_sdk::serde::Deserialize,
            ::calimero_sdk::borsh::BorshSerialize,
            ::calimero_sdk::borsh::BorshDeserialize,
        )]
        #[borsh(crate = "::calimero_sdk::borsh")]
        #[serde(crate = "::calimero_sdk::serde")]
        #[repr(transparent)]
        $($vis)* struct $name($crate::types::id::Id< $len $(, $str)? >);

        impl $name {
            pub const fn new(id: [u8; $len]) -> Self {
                type DefinedId = $crate::types::id::Id::< $len $(, $str)? >;

                let _guard = DefinedId::SIZE_GUARD;

                Self($crate::types::id::Id::new(id))
            }
        }

        impl ::core::ops::Deref for $name {
            type Target = $crate::types::id::Id< $len $(, $str)? >;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::core::convert::AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }

        impl ::core::convert::From<[u8; $len]> for $name {
            fn from(id: [u8; $len]) -> Self {
                Self::new(id)
            }
        }

        // SAFETY: the macro guarantees the newtype is the same
        //         size as Id<N, _>
        unsafe impl $crate::types::id::IdTransmute<$len> for $name {}
    };
}

pub(crate) use define;
