use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{sleep, Sleep};

#[pin_project]
pub struct MaybeSleep {
    duration: Option<Duration>,
    #[pin]
    sleep: Option<Sleep>,
}

impl MaybeSleep {
    pub fn new(duration: Option<Duration>) -> Self {
        Self {
            duration,
            sleep: duration.map(sleep),
        }
    }
}

impl Future for MaybeSleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if this.sleep.is_some() {
            let sleep = this.sleep.as_pin_mut().unwrap();
            sleep.poll(cx)
        } else {
            Poll::Pending
        }
    }
}
