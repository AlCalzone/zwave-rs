pub mod oneshot;

#[cfg(feature = "std")]
mod std_impl;
#[cfg(feature = "std")]
pub use std_impl::*;

#[cfg(feature = "embassy")]
mod embassy_impl;
#[cfg(feature = "embassy")]
pub use embassy_impl::*;
