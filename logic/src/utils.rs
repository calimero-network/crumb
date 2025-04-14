use std::borrow::Cow;
use std::convert::Infallible;

use calimero_sdk::app;

pub trait IntoResult<T> {
    type Error;

    fn into_result(self) -> Result<T, Self::Error>;
}

impl<T> IntoResult<T> for T {
    type Error = Infallible;

    fn into_result(self) -> Result<T, Self::Error> {
        Ok(self)
    }
}

impl<T, E> IntoResult<T> for Result<T, E> {
    type Error = E;

    fn into_result(self) -> Result<T, Self::Error> {
        self
    }
}

pub fn unique<T, R>(factory: impl Fn() -> T, test: impl Fn(&T) -> R) -> app::Result<T>
where
    R: IntoResult<bool>,
    R::Error: core::error::Error,
{
    for _ in 0..10 {
        let value = factory();
        if !test(&value).into_result()? {
            return Ok(value);
        }
    }

    app::bail!(
        "unable to determine unique value for `{}` after 10 attempts",
        std::any::type_name::<T>()
    );
}

pub fn truncate_string(s: &str, max_width: usize) -> Cow<'_, str> {
    if s.len() <= max_width {
        return s.into();
    }
    s.chars().take(max_width - 1).chain(Some('…')).collect()
}
