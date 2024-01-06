use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{sleep, Sleep};

#[pin_project]
pub struct MaybeSleep {
    duration: Option<Duration>,
    #[pin]
    sleep: Option<Sleep>,
}

impl MaybeSleep {
    pub fn new(duration: Option<Duration>) -> Self {
        Self {
            duration,
            sleep: duration.map(sleep),
        }
    }
}

impl Future for MaybeSleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if this.sleep.is_some() {
            let sleep = this.sleep.as_pin_mut().unwrap();
            sleep.poll(cx)
        } else {
            Poll::Pending
        }
    }
}

pub fn now() -> String {
    use time::{macros::format_description, OffsetDateTime};
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

/// Provides the `to_discriminant` and `from_discriminant` method for enums implementing this trait.
///
/// # Safety
/// The implementer must ensure that the enum's `#[repr(...)]` matches the generic type of this trait.
/// For example, an enum implementing `ToDiscriminant<u8>` MUST be marked with `#[repr(u8)]`.
pub unsafe trait Discriminant<T: Copy> {
    fn to_discriminant(&self) -> T {
        // SAFETY: Because `Self` is marked `repr(<T>)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `T` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<T>() }
    }

    /// Constructs a fieldless enum variant from its discriminant.
    /// # Safety
    /// The caller must ensure that the given discriminant matches a fieldless variant of the enum.
    unsafe fn from_discriminant(d: &T) -> Self
    where
        Self: Sized + Copy,
    {
        *<*const _>::from(&d).cast::<Self>()
    }
}
