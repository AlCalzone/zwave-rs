// =============================================================================
// Locked<T>
// =============================================================================

#[cfg(feature = "std")]
pub struct Locked<T> {
    inner: std::sync::RwLock<T>,
}

#[cfg(feature = "std")]
impl<T> Locked<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: std::sync::RwLock::new(value),
        }
    }

    pub fn inspect<R>(&self, inspect: impl FnOnce(&T) -> R) -> R {
        let guard = self
            .inner
            .read()
            .expect("failed to lock storage for reading");
        inspect(&guard)
    }

    pub fn update<R>(&self, update: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self
            .inner
            .write()
            .expect("failed to lock storage for writing");
        update(&mut guard)
    }

    pub fn set(&self, value: T) {
        self.update(|slot| *slot = value);
    }

    pub fn replace(&self, value: T) -> T {
        self.update(|slot| core::mem::replace(slot, value))
    }
}

#[cfg(feature = "std")]
impl<T: Copy> Locked<T> {
    pub fn get(&self) -> T {
        self.inspect(|value| *value)
    }
}

#[cfg(feature = "std")]
impl<T: Clone> Locked<T> {
    pub fn cloned(&self) -> T {
        self.inspect(Clone::clone)
    }
}

#[cfg(feature = "embassy")]
pub struct Locked<T> {
    inner: embassy_sync::blocking_mutex::Mutex<
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        core::cell::RefCell<T>,
    >,
}

#[cfg(feature = "embassy")]
impl<T> Locked<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: embassy_sync::blocking_mutex::Mutex::new(core::cell::RefCell::new(value)),
        }
    }

    pub fn inspect<R>(&self, inspect: impl FnOnce(&T) -> R) -> R {
        self.inner.lock(|cell| inspect(&cell.borrow()))
    }

    pub fn update<R>(&self, update: impl FnOnce(&mut T) -> R) -> R {
        self.inner.lock(|cell| update(&mut cell.borrow_mut()))
    }

    pub fn set(&self, value: T) {
        self.update(|slot| *slot = value);
    }

    pub fn replace(&self, value: T) -> T {
        self.update(|slot| core::mem::replace(slot, value))
    }
}

#[cfg(feature = "embassy")]
impl<T: Copy> Locked<T> {
    pub fn get(&self) -> T {
        self.inspect(|value| *value)
    }
}

#[cfg(feature = "embassy")]
impl<T: Clone> Locked<T> {
    pub fn cloned(&self) -> T {
        self.inspect(Clone::clone)
    }
}

// =============================================================================
// OnceLock<T>
// =============================================================================

#[cfg(feature = "std")]
pub use std::sync::OnceLock;

#[cfg(feature = "embassy")]
pub use embassy_sync::once_lock::OnceLock;

// =============================================================================
// Mutex<T> — closure-based API that works on both platforms
// =============================================================================

/// A mutex with a closure-based access pattern that works on both std and no_std.
///
/// Usage:
/// ```ignore
/// let mutex = Mutex::new(vec![1, 2, 3]);
/// mutex.lock(|vec| vec.push(4));
/// let len = mutex.lock(|vec| vec.len());
/// ```
#[cfg(feature = "std")]
pub struct Mutex<T> {
    inner: std::sync::Mutex<T>,
}

#[cfg(feature = "std")]
impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: std::sync::Mutex::new(value),
        }
    }

    pub fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.inner.lock().expect("Mutex poisoned");
        f(&mut guard)
    }
}

#[cfg(feature = "std")]
impl<T: Default> Default for Mutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[cfg(feature = "embassy")]
pub struct Mutex<T> {
    inner: embassy_sync::blocking_mutex::Mutex<
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        core::cell::RefCell<T>,
    >,
}

#[cfg(feature = "embassy")]
impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: embassy_sync::blocking_mutex::Mutex::new(core::cell::RefCell::new(value)),
        }
    }

    pub fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.inner.lock(|cell| f(&mut cell.borrow_mut()))
    }
}

#[cfg(feature = "embassy")]
impl<T: Default> Default for Mutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
