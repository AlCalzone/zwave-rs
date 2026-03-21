use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

// =============================================================================
// Instant
// =============================================================================

#[cfg(feature = "std")]
pub use std::time::Instant;

#[cfg(feature = "embassy")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(embassy_time::Instant);

#[cfg(feature = "embassy")]
impl Instant {
    pub fn now() -> Self {
        Self(embassy_time::Instant::now())
    }

    pub fn checked_add(&self, d: Duration) -> Option<Self> {
        let micros = u64::try_from(d.as_micros()).ok()?;
        let ticks = embassy_time::Duration::from_micros(micros);
        Some(Self(self.0 + ticks))
    }

    pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
        if self.0 < earlier.0 {
            return None;
        }
        let diff = self.0 - earlier.0;
        Some(Duration::from_micros(diff.as_micros()))
    }
}

#[cfg(feature = "embassy")]
impl core::ops::Add<Duration> for Instant {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self {
        self.checked_add(rhs).expect("overflow when adding duration to instant")
    }
}

#[cfg(feature = "embassy")]
impl core::ops::Sub for Instant {
    type Output = Duration;
    fn sub(self, rhs: Self) -> Duration {
        self.checked_duration_since(rhs)
            .expect("overflow when subtracting instants")
    }
}

// =============================================================================
// Timestamp
// =============================================================================

#[cfg(feature = "std")]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);

#[cfg(feature = "std")]
impl Timestamp {
    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        )
    }
}

#[cfg(feature = "std")]
impl Clone for Timestamp {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

#[cfg(feature = "embassy")]
pub struct Timestamp(embassy_time::Instant);

#[cfg(feature = "embassy")]
impl Timestamp {
    pub fn now() -> Self {
        Self(embassy_time::Instant::now())
    }
}

#[cfg(feature = "embassy")]
impl core::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let millis = self.0.as_millis();
        write!(f, "{}.{:03}s", millis / 1000, millis % 1000)
    }
}

#[cfg(feature = "embassy")]
impl Clone for Timestamp {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

// =============================================================================
// Timer
// =============================================================================

#[cfg(feature = "std")]
pub struct Timer(futures_timer::Delay);

#[cfg(feature = "embassy")]
pub struct Timer(embassy_time::Timer);

#[cfg(any(feature = "std", feature = "embassy"))]
impl Timer {
    pub fn after(duration: Duration) -> Self {
        #[cfg(feature = "std")]
        {
            Self(futures_timer::Delay::new(duration))
        }
        #[cfg(feature = "embassy")]
        {
            Self(embassy_time::Timer::after(embassy_time::Duration::from_micros(
                duration.as_micros() as u64,
            )))
        }
    }
}

#[cfg(any(feature = "std", feature = "embassy"))]
impl Future for Timer {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        #[cfg(feature = "std")]
        {
            // SAFETY: Delay is Unpin
            Pin::new(&mut self.get_mut().0).poll(cx)
        }
        #[cfg(feature = "embassy")]
        {
            // SAFETY: embassy Timer is Unpin
            Pin::new(&mut self.get_mut().0).poll(cx)
        }
    }
}

// =============================================================================
// MaybeSleep
// =============================================================================

#[cfg(any(feature = "std", feature = "embassy"))]
pub struct MaybeSleep {
    sleep: Option<Timer>,
}

#[cfg(any(feature = "std", feature = "embassy"))]
impl MaybeSleep {
    pub fn new(duration: Option<Duration>) -> Self {
        Self {
            sleep: duration.map(Timer::after),
        }
    }
}

#[cfg(any(feature = "std", feature = "embassy"))]
impl Future for MaybeSleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.get_mut().sleep {
            Some(timer) => Pin::new(timer).poll(cx),
            None => Poll::Pending,
        }
    }
}
