use std::sync::Mutex;
use tokio::sync::oneshot;
use unique_id::sequence::SequenceGenerator;
use unique_id::Generator;

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
    pub fn add(&self, predicate: Predicate<T>) -> AwaitedRef<T> {
        let (tx, rx) = oneshot::channel::<(T, oneshot::Sender<()>)>();
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
        AwaitedRef::new(id, self, rx)
    }

    /// Finds the first entry in the registry that matches the given value, returning the channel
    /// that can be used to receive the value when it is available.
    /// The entry is removed from the registry.
    pub fn take_matching(&self, value: &T) -> Option<oneshot::Sender<(T, oneshot::Sender<()>)>> {
        let mut vec = self.store.lock().unwrap();
        let index = vec.iter().position(|a| (a.predicate)(value));
        index.map(|i| vec.remove(i).channel)
    }

    /// Removes an entry from the registry using the given `AwaitedRef`.
    pub fn remove(&self, awaited: &AwaitedRef<T>) {
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
    pub channel: oneshot::Sender<(T, oneshot::Sender<()>)>,
}

pub struct AwaitedRef<'a, T> {
    id: i64,
    registry: &'a AwaitedRegistry<T>,
    channel: Option<oneshot::Receiver<(T, oneshot::Sender<()>)>>,
}

impl<'a, T> AwaitedRef<'a, T> {
    pub fn new(id: i64, registry: &'a AwaitedRegistry<T>, channel: oneshot::Receiver<(T, oneshot::Sender<()>)>) -> Self {
        Self {
            id,
            registry,
            channel: Some(channel),
        }
    }

    pub fn take_channel(&mut self) -> oneshot::Receiver<(T, oneshot::Sender<()>)> {
        self.channel.take().unwrap()
    }
}

impl<T> Drop for AwaitedRef<'_, T> {
    fn drop(&mut self) {
        self.registry.remove(self);
    }
}
