use crate::error::{Error, Result};
use zwave_pal::prelude::*;
use core::sync::atomic::{AtomicI64, Ordering};
use zwave_pal::time::MaybeSleep;
use zwave_pal::channel::oneshot;

pub type Predicate<T> = Box<dyn Fn(&T) -> bool + Sync + Send>;

/// A registry of `Awaited` values, each of which is associated with a predicate that determines
/// whether a given value matches the awaited value.
///
/// Adding an entry hands out an `AwaitedRef`, which is used to receive the value when it is
/// available. The `AwaitedRef` is automatically removed from the registry when it is dropped.
pub struct AwaitedRegistry<T> {
    next_id: AtomicI64,
    store: zwave_pal::sync::Mutex<Vec<Awaited<T>>>,
}

impl<T> Default for AwaitedRegistry<T> {
    fn default() -> Self {
        Self {
            next_id: AtomicI64::new(0),
            store: zwave_pal::sync::Mutex::default(),
        }
    }
}

impl<T> AwaitedRegistry<T> {
    /// Adds an entry to the registry with a given predicate, returning an `AwaitedRef` that can be
    /// used to receive the value when it is available.
    pub fn add(
        self: &Arc<Self>,
        predicate: Predicate<T>,
        timeout: Option<core::time::Duration>,
    ) -> AwaitedRef<T> {
        let (tx, rx) = oneshot::channel::<T>();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let awaited = Awaited {
            id,
            predicate,
            channel: tx,
        };
        self.store.lock(|vec| vec.push(awaited));
        AwaitedRef::new(id, self.clone(), timeout, rx)
    }

    /// Finds the first entry in the registry that matches the given value, returning the channel
    /// that can be used to receive the value when it is available.
    /// The entry is removed from the registry.
    pub fn take_matching(self: &Arc<Self>, value: &T) -> Option<oneshot::Sender<T>> {
        self.store.lock(|vec| {
            let index = vec.iter().position(|a| (a.predicate)(value));
            index.map(|i| vec.remove(i).channel)
        })
    }

    /// Removes an entry from the registry using the given `AwaitedRef`.
    pub fn remove(self: &Arc<Self>, awaited: &AwaitedRef<T>) {
        self.store.lock(|vec| {
            vec.retain(|a| a.id != awaited.id);
        });
    }
}

pub struct Awaited<T> {
    pub id: i64,
    pub predicate: Predicate<T>,
    pub channel: oneshot::Sender<T>,
}

pub struct AwaitedRef<T> {
    id: i64,
    registry: Arc<AwaitedRegistry<T>>,
    timeout: Option<core::time::Duration>,
    channel: Option<oneshot::Receiver<T>>,
}

impl<T> AwaitedRef<T> {
    pub fn new(
        id: i64,
        registry: Arc<AwaitedRegistry<T>>,
        timeout: Option<core::time::Duration>,
        channel: oneshot::Receiver<T>,
    ) -> Self {
        Self {
            id,
            registry,
            timeout,
            channel: Some(channel),
        }
    }

    /// Begins awaiting the value
    pub async fn try_await(mut self) -> Result<T> {
        let sleep = MaybeSleep::new(self.timeout);
        let receiver = self
            .channel
            .take()
            .expect("try_await may only be called once");
        zwave_pal::select_biased! {
            result = receiver => result.map_err(|_| Error::Internal),
            _ = sleep => Err(Error::Timeout),
        }
    }
}

impl<T> core::fmt::Debug for AwaitedRef<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AwaitedRef").field("id", &self.id).finish()
    }
}

impl<T> Drop for AwaitedRef<T> {
    fn drop(&mut self) {
        self.registry.remove(self);
    }
}
