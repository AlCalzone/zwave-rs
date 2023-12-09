use num_traits::{Bounded, One, Unsigned, WrappingAdd};

/// A counter that starts at 1 and wraps after surpassing the maximum value of its type or the specified maximum.
pub struct WrappingCounter<T>
where
    T: Bounded + Ord + Unsigned + WrappingAdd + One + Copy,
{
    value: T,
    max: Option<T>,
}

impl<T> Default for WrappingCounter<T>
where
    T: Bounded + Ord + Unsigned + WrappingAdd + One + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> WrappingCounter<T>
where
    T: Bounded + Ord + Unsigned + WrappingAdd + One + Copy,
{
    pub fn new() -> Self {
        Self {
            value: T::zero(),
            max: None,
        }
    }

    pub fn new_with_max(max: T) -> Self {
        Self {
            value: T::zero(),
            max: Some(max),
        }
    }

    pub fn increment(&mut self) -> T {
        let mut next = self.value.wrapping_add(&T::one());
        next = match self.max {
            Some(max) if next > max => T::one(),
            _ => next,
        };
        if next.is_zero() {
            next = T::one();
        }

        self.value = next;
        self.value
    }
}

#[test]
fn test_increment() {
    let mut counter = WrappingCounter::new_with_max(5u8);
    assert_eq!(counter.increment(), 1);
    assert_eq!(counter.increment(), 2);
    assert_eq!(counter.increment(), 3);
    assert_eq!(counter.increment(), 4);
    assert_eq!(counter.increment(), 5);
    assert_eq!(counter.increment(), 1);

    let mut counter = WrappingCounter::new_with_max(4usize);
    assert_eq!(counter.increment(), 1);
    assert_eq!(counter.increment(), 2);
    assert_eq!(counter.increment(), 3);
    assert_eq!(counter.increment(), 4);
    assert_eq!(counter.increment(), 1);
}
