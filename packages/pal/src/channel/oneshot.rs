#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Canceled;

impl core::fmt::Display for Canceled {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "oneshot canceled")
    }
}

// =============================================================================
// std backend
// =============================================================================

#[cfg(feature = "std")]
pub use std_impl::*;

#[cfg(feature = "std")]
mod std_impl {
    use super::Canceled;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use futures::channel::oneshot as futures_oneshot;

    pub struct Sender<T> {
        inner: futures_oneshot::Sender<T>,
    }

    impl<T> Sender<T> {
        pub fn send(self, value: T) -> Result<(), T> {
            self.inner.send(value)
        }
    }

    pub struct Receiver<T> {
        inner: futures_oneshot::Receiver<T>,
    }

    impl<T> Future for Receiver<T> {
        type Output = Result<T, Canceled>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let inner = &mut self.get_mut().inner;
            Pin::new(inner).poll(cx).map(|r| r.map_err(|_| Canceled))
        }
    }

    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = futures_oneshot::channel();
        (Sender { inner: tx }, Receiver { inner: rx })
    }
}

// =============================================================================
// embassy backend — uses Signal as a single-value oneshot
// =============================================================================

#[cfg(feature = "embassy")]
pub use embassy_impl::*;

#[cfg(feature = "embassy")]
mod embassy_impl {
    use super::Canceled;
    use alloc::sync::Arc;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;

    type SharedSignal<T> = Arc<Signal<CriticalSectionRawMutex, T>>;

    pub struct Sender<T> {
        inner: SharedSignal<T>,
    }

    impl<T> Sender<T> {
        pub fn send(self, value: T) -> Result<(), T> {
            self.inner.signal(value);
            Ok(())
        }
    }

    /// A oneshot receiver backed by an embassy `Signal`.
    ///
    /// Unlike the std backend, this cannot detect sender disconnection —
    /// if the sender is dropped without sending, this future will pend forever.
    /// Callers should always use this with a timeout via `select_biased!`.
    pub struct Receiver<T> {
        inner: SharedSignal<T>,
    }

    impl<T> Future for Receiver<T> {
        type Output = Result<T, Canceled>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.inner
                .poll_wait(cx)
                .map(Ok)
        }
    }

    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let signal = Arc::new(Signal::new());
        (
            Sender { inner: signal.clone() },
            Receiver { inner: signal },
        )
    }
}
