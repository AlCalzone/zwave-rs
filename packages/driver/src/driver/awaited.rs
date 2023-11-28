use crate::error::{Error, Result};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;
use unique_id::sequence::SequenceGenerator;
use unique_id::Generator;
use zwave_core::util::MaybeSleep;

pub type Predicate<T> = Box<dyn Fn(&T) -> bool + Sync + Send>;

/// A registry of `Awaited` values, each of which is associated with a predicate that determines
/// whether a given value matches the awaited value.
///
/// Adding an entry hands out an `AwaitedRef`, which is used to receive the value when it is
/// available. The `AwaitedRef` is automatically removed from the registry when it is dropped.
pub struct AwaitedRegistry<T> {
    // TODO: Consider using something that does not use global state
    sequence_gen: SequenceGenerator,
    store: Mutex<Vec<Awaited<T>>>,
}

impl<T> Default for AwaitedRegistry<T> {
    fn default() -> Self {
        Self {
            sequence_gen: SequenceGenerator,
            store: Mutex::default(),
        }
    }
}

impl<T> AwaitedRegistry<T> {
    /// Adds an entry to the registry with a given predicate, returning an `AwaitedRef` that can be
    /// used to receive the value when it is available.
    pub fn add(
        self: &Arc<Self>,
        predicate: Predicate<T>,
        timeout: Option<Duration>,
    ) -> AwaitedRef<T> {
        let (tx, rx) = oneshot::channel::<T>();
        let id = self.sequence_gen.next_id();
        let awaited = Awaited {
            id,
            predicate,
            channel: tx,
        };
        {
            let mut vec = self.store.lock().unwrap();
            vec.push(awaited);
        }
        AwaitedRef::new(id, self.clone(), timeout, rx)
    }

    /// Finds the first entry in the registry that matches the given value, returning the channel
    /// that can be used to receive the value when it is available.
    /// The entry is removed from the registry.
    pub fn take_matching(
        self: &Arc<Self>,
        value: &T,
    ) -> Option<oneshot::Sender<T>> {
        let mut vec = self.store.lock().unwrap();
        let index = vec.iter().position(|a| (a.predicate)(value));
        index.map(|i| vec.remove(i).channel)
    }

    /// Removes an entry from the registry using the given `AwaitedRef`.
    pub fn remove(self: &Arc<Self>, awaited: &AwaitedRef<T>) {
        let mut vec = self.store.lock().unwrap();
        vec.retain(|a| a.id != awaited.id);
    }

    // pub fn len(&self) -> usize {
    //     self.store.lock().unwrap().len()
    // }
}

pub struct Awaited<T> {
    pub id: i64,
    pub predicate: Predicate<T>,
    pub channel: oneshot::Sender<T>,
}

pub struct AwaitedRef<T> {
    id: i64,
    registry: Arc<AwaitedRegistry<T>>,
    timeout: Option<Duration>,
    channel: Option<oneshot::Receiver<T>>,
}

impl<T> AwaitedRef<T> {
    pub fn new(
        id: i64,
        registry: Arc<AwaitedRegistry<T>>,
        timeout: Option<Duration>,
        channel: oneshot::Receiver<T>,
    ) -> Self {
        Self {
            id,
            registry,
            timeout,
            channel: Some(channel),
        }
    }

    fn take_channel(&mut self) -> oneshot::Receiver<T> {
        self.channel.take().unwrap()
    }

    /// Begins awaiting the value
    pub async fn try_await(mut self) -> Result<T> {
        let sleep = MaybeSleep::new(self.timeout);
        tokio::select! {
            // We pass the entire result including the oneshot channel to the caller,
            // so that they can acknowledge the command when they handled it. This avoids
            // race conditions where the driver may attempt to handle the next serial frame
            // before it is expected.
            result = self.take_channel() => result.map_err(|_| Error::Internal),
            _ = sleep => Err(Error::Timeout),
        }
    }
}

impl<T> Drop for AwaitedRef<T> {
    fn drop(&mut self) {
        self.registry.remove(self);
    }
}
