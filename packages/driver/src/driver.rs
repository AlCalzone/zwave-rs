use crate::error::Result;
use crate::serial_api::SerialApi;
use crate::LogSender;
use awaited::Predicate;
use futures::channel::{mpsc, oneshot};
use typed_builder::TypedBuilder;
use std::sync::Arc;
use std::time::{Duration, Instant};
use storage::DriverStorage;
use zwave_cc::prelude::*;
use zwave_core::log::Loglevel;
use zwave_core::submodule;
use zwave_logging::LogInfo;
use zwave_serial::prelude::*;

pub(crate) mod awaited;
pub(crate) mod cache;
mod storage;

submodule!(exec_controller_command);
submodule!(controller_commands);
submodule!(exec_node_command);
submodule!(actor);
submodule!(handle);
// submodule!(node_api);
// submodule!(node_commands);

#[derive(Clone)]
pub struct Driver {
    cmd_tx: DriverInputSender,
    serial_api: SerialApi,
    pub(crate) storage: Arc<DriverStorage>,
}

pub struct DriverActor {
    // Channels to interact with this actor
    log_queue: LogSender,
    input_tx: DriverInputSender,
    input_rx: DriverInputReceiver,
    event_tx: DriverEventSender,

    // Handles to lower layers
    serial_api: SerialApi,

    /// Storage shared between this actor and its API handles
    storage: Arc<DriverStorage>,

    security_keys: SecurityKeys,
    awaited_ccs: Vec<AwaitedCC>,
}

pub struct DriverAdapter {
    pub input_tx: DriverInputSender,
    pub event_rx: DriverEventReceiver,
}

impl Driver {
    pub fn new(serial_api: &SerialApi, log_tx: LogSender, security_keys: SecurityKeys) -> (Self, DriverActor, DriverAdapter) {
        let (input_tx, input_rx) = mpsc::channel(16);
        let (event_tx, event_rx) = mpsc::channel(16);

        let storage = Arc::new(DriverStorage::new());

        let driver = Driver {
            cmd_tx: input_tx.clone(),
            serial_api: serial_api.clone(),
            storage: storage.clone(),
        };

        let adapter = DriverAdapter {
            input_tx: input_tx.clone(),
            event_rx,
        };

        let actor = DriverActor {
            log_queue: log_tx,
            input_tx,
            input_rx,
            event_tx,
            serial_api: serial_api.clone(),
            storage,
            security_keys,
            awaited_ccs: Vec::new(),
        };

        (driver, actor, adapter)
    }
}

pub enum DriverInput {
    /// An unsolicited command needs to be handled
    Unsolicited { command: Command },
    /// Log the given message
    Log { log: LogInfo, level: Loglevel },
    /// Initialize the security managers
    InitSecurityManagers,
    /// Waits for a CC matching the given predicate
    AwaitCC {
        predicate: Predicate<WithAddress<CC>>,
        timeout: Option<Duration>,
        // FIXME: Make this a specific result type
        callback: oneshot::Sender<Result<WithAddress<CC>>>,
    },
}

pub enum DriverEvent {
    // FIXME: Add command to forward unhandled commands to the application
}

type DriverInputSender = mpsc::Sender<DriverInput>;
type DriverInputReceiver = mpsc::Receiver<DriverInput>;

type DriverEventSender = mpsc::Sender<DriverEvent>;
type DriverEventReceiver = mpsc::Receiver<DriverEvent>;

struct AwaitedCC {
    timeout: Option<Instant>,
    predicate: Predicate<WithAddress<CC>>,
    callback: oneshot::Sender<Result<WithAddress<CC>>>,
}

#[derive(TypedBuilder)]
pub struct DriverOptions {
    #[builder(default)]
    security_keys: SecurityKeys,
}

#[derive(Default, Clone, TypedBuilder)]
pub struct SecurityKeys {
    #[builder(default, setter(into))]
    pub s0_legacy: Option<Vec<u8>>,
}
