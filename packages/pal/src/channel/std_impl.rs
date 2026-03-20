use futures::channel::mpsc;
use futures::StreamExt;

pub struct Sender<T> {
    inner: mpsc::Sender<T>,
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
        self.inner
            .clone()
            .try_send(item)
            .map_err(|e| TrySendError {
                disconnected: e.is_disconnected(),
                item: e.into_inner(),
            })
    }
}

pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
}

impl<T> Receiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        self.inner.next().await
    }
}

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::channel(capacity);
    (Sender { inner: tx }, Receiver { inner: rx })
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
