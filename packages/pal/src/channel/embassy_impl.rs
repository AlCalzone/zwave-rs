use alloc::sync::Arc;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

// Embassy channels have const-generic capacity. We use a fixed reasonable capacity
// and allocate on the heap via Arc so Sender/Receiver can be cloned/owned.
const DEFAULT_CAPACITY: usize = 16;

pub struct Sender<T> {
    inner: Arc<Channel<CriticalSectionRawMutex, T, DEFAULT_CAPACITY>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Sender<T> {
    pub fn try_send(&self, item: T) -> Result<(), TrySendError<T>> {
        self.inner.try_send(item).map_err(|e| match e {
            embassy_sync::channel::TrySendError::Full(item) => TrySendError {
                disconnected: false,
                item,
            },
        })
    }
}

pub struct Receiver<T> {
    inner: Arc<Channel<CriticalSectionRawMutex, T, DEFAULT_CAPACITY>>,
}

impl<T> Receiver<T> {
    /// Receives the next value from the channel.
    ///
    /// Note: Unlike the std backend, this will never return `None` when all senders
    /// are dropped — embassy's Channel has no disconnect detection. Our actor loops
    /// run indefinitely, so this is acceptable. If termination is needed in the future,
    /// send an explicit shutdown message through the channel.
    pub async fn recv(&mut self) -> Option<T> {
        Some(self.inner.receive().await)
    }
}

/// Creates a new channel.
///
/// Embassy channels require capacity as a const generic, so the runtime
/// `capacity` parameter cannot be forwarded. The channel is always created
/// with `DEFAULT_CAPACITY` (16). A debug assertion ensures callers don't
/// accidentally request a different size.
pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    debug_assert_eq!(
        capacity, DEFAULT_CAPACITY,
        "embassy backend uses a fixed channel capacity of {DEFAULT_CAPACITY}, got {capacity}"
    );
    let ch = Arc::new(Channel::new());
    (Sender { inner: ch.clone() }, Receiver { inner: ch })
}

pub struct TrySendError<T> {
    disconnected: bool,
    item: T,
}

impl<T> TrySendError<T> {
    pub fn is_disconnected(&self) -> bool {
        self.disconnected
    }

    pub fn into_inner(self) -> T {
        self.item
    }
}

impl<T> core::fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TrySendError")
            .field("disconnected", &self.disconnected)
            .finish()
    }
}

impl<T> core::fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.disconnected {
            write!(f, "channel disconnected")
        } else {
            write!(f, "channel full")
        }
    }
}
