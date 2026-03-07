use futures_timer::Delay;
use std::borrow::Cow;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use unicode_segmentation::UnicodeSegmentation;

pub struct MaybeSleep {
    sleep: Option<Delay>,
}

impl MaybeSleep {
    pub fn new(duration: Option<Duration>) -> Self {
        Self {
            sleep: duration.map(Delay::new),
        }
    }
}

impl Future for MaybeSleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.get_mut().sleep {
            Some(delay) => Pin::new(delay).poll(cx),
            None => Poll::Pending,
        }
    }
}

pub fn now() -> String {
    use time::{OffsetDateTime, macros::format_description};
    let format =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:4]Z");
    OffsetDateTime::now_utc().format(format).unwrap()
}

#[macro_export]
macro_rules! submodule {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

/// Provides the `to_discriminant` method for enums implementing this trait.
///
/// # Safety
/// The implementer must ensure that the enum's `#[repr(...)]` matches the generic type of this trait.
/// For example, an enum implementing `ToDiscriminant<u8>` MUST be marked with `#[repr(u8)]`.
pub unsafe trait ToDiscriminant<T: Copy> {
    fn to_discriminant(&self) -> T {
        // SAFETY: Because `Self` is marked `repr(<T>)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `T` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<T>() }
    }
}

pub fn str_width(string: &str) -> usize {
    string.graphemes(true).count()
}

pub fn to_lines(text: impl Into<Cow<'static, str>>) -> Vec<Cow<'static, str>> {
    let text = text.into();
    if text.is_empty() {
        // Return at least one empty line
        return vec!["".into()];
    }

    text.lines().map(|line| line.to_owned().into()).collect()
}

#[macro_export]
macro_rules! hex_literal {
    ($hex:expr) => {
        hex::decode($hex).unwrap()
    };
}

#[macro_export]
macro_rules! hex_bytes {
    ($hex:expr) => {
        bytes::BytesMut::from(hex::decode($hex).unwrap().as_slice()).freeze()
    };
}

#[macro_export]
macro_rules! hex_bytes_mut {
    ($hex:expr) => {
        bytes::BytesMut::from(hex::decode($hex).unwrap().as_slice())
    };
}
