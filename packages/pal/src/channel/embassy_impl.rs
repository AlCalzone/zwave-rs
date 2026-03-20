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
    pub async fn recv(&mut self) -> Option<T> {
        Some(self.inner.receive().await)
    }
}

pub fn channel<T>(_capacity: usize) -> (Sender<T>, Receiver<T>) {
    let ch = Arc::new(Channel::new());
    (
        Sender { inner: ch.clone() },
        Receiver { inner: ch },
    )
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
