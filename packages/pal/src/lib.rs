#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "embassy")))]
compile_error!("zwave-pal requires either the `std` or `embassy` feature to be enabled");

#[cfg(all(feature = "std", feature = "embassy"))]
compile_error!("zwave-pal features `std` and `embassy` are mutually exclusive");

extern crate alloc;

pub mod channel;
pub mod select;
pub mod sync;
pub mod time;

/// Alloc types that the std prelude provides but `extern crate alloc` does not.
/// In std builds the module is empty (the std prelude already provides these).
/// Library crates import this unconditionally via `use zwave_pal::prelude::*`.
/// Common alloc types and macros.
///
/// In std builds, most of these are redundant (already in the std prelude),
/// but re-exporting them unconditionally means downstream no_std crates get
/// them regardless of whether their own `std` feature is enabled.
pub mod prelude {
    pub use alloc::{
        borrow::{Cow, ToOwned},
        boxed::Box,
        format,
        string::{String, ToString},
        sync::Arc,
        vec,
        vec::Vec,
    };
}

/// Re-export `getrandom` so downstream crates and applications can use it
/// without adding a direct dependency. On embassy builds, the `custom` feature
/// is enabled — applications must call `zwave_pal::rng::register_custom_getrandom!`
/// to provide an RNG implementation.
pub use getrandom as rng;

// Re-exports needed by the select_biased! macro.
// These must be public so the macro can reference them from downstream crates.
#[cfg(feature = "std")]
#[doc(hidden)]
pub use futures as __reexport_futures;

#[cfg(feature = "embassy")]
#[doc(hidden)]
pub use embassy_futures as __reexport_embassy_futures;
