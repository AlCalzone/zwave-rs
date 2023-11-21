use std::sync::Mutex;

use tokio::sync::oneshot;
use zwave_serial::command::Command;

pub type AwaitedCommandRegistry = Mutex<Vec<AwaitedCommand>>;

pub struct AwaitedCommand {
    pub id: i64,
    pub predicate: fn(&Command) -> bool,
    pub channel: oneshot::Sender<Command>,
}

pub struct AwaitedCommandRef<'a> {
    id: i64,
    registry: &'a AwaitedCommandRegistry,
    channel: Option<oneshot::Receiver<Command>>,
}

impl<'a> AwaitedCommandRef<'a> {
    pub fn new(
        id: i64,
        registry: &'a AwaitedCommandRegistry,
        channel: oneshot::Receiver<Command>,
    ) -> Self {
        Self {
            id,
            registry,
            channel: Some(channel),
        }
    }

    pub fn take_channel(&mut self) -> oneshot::Receiver<Command> {
        self.channel.take().unwrap()
    }
}

impl Drop for AwaitedCommandRef<'_> {
    fn drop(&mut self) {
        let mut registry = self.registry.lock().unwrap();
        registry.retain(|a| a.id != self.id);
    }
}
