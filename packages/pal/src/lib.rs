#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod channel;
pub mod select;
pub mod sync;
pub mod time;

// Re-exports needed by the select_biased! macro.
// These must be public so the macro can reference them from downstream crates.
#[cfg(feature = "std")]
#[doc(hidden)]
pub use futures as __reexport_futures;

#[cfg(feature = "embassy")]
#[doc(hidden)]
pub use embassy_futures as __reexport_embassy_futures;
