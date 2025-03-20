use crate::error::Result;
use crate::{LogReceiver, LogSender};
use futures::channel::{mpsc, oneshot};
use storage::SerialApiStorage;
use std::sync::Arc;
use std::time::Instant;
use zwave_core::log::Loglevel;
use zwave_core::prelude::*;
use zwave_core::submodule;
use zwave_core::wrapping_counter::WrappingCounter;
use zwave_logging::LogInfo;
use zwave_serial::frame::{RawSerialFrame, SerialFrame};
use zwave_serial::prelude::*;

submodule!(serial_api_machine);
submodule!(handle);
submodule!(actor);
mod storage;

type SerialFrameReceiver = mpsc::Receiver<RawSerialFrame>;
type SerialFrameSender = mpsc::Sender<RawSerialFrame>;

type SerialApiInputSender = mpsc::Sender<SerialApiInput>;
type SerialApiInputReceiver = mpsc::Receiver<SerialApiInput>;

type SerialApiEventSender = mpsc::Sender<SerialApiEvent>;
type SerialApiEventReceiver = mpsc::Receiver<SerialApiEvent>;

pub trait ExecutableCommand: CommandRequest + AsCommandRaw {}
impl<T> ExecutableCommand for T where T: CommandRequest + AsCommandRaw {}

struct SerialApiCommandState {
    command: Box<dyn ExecutableCommand>,
    timeout: Option<Instant>,
    expects_response: bool,
    expects_callback: bool,
    machine: SerialApiMachine,
    callback: Option<oneshot::Sender<Result<SerialApiMachineResult>>>,
}

/// An actor to interact with the Serial API in a sans-io fashion:
/// - serial frames must be sent to and read from the driver
/// - logs must be read from the driver and handled outside
/// - inputs must be sent to the driver
///
/// It does not store any cached information about the network, except for what it needs to
/// correctly serialize and deserialize commands. If any state needs to be kept aside from that,
/// the relevant abstractions must handle this themselves.
pub struct SerialApiActor {
    // Channels to interact with this actor
    serial_in: SerialFrameReceiver,
    serial_out: SerialFrameSender,
    log_queue: LogSender,
    input_tx: SerialApiInputSender,
    input_rx: SerialApiInputReceiver,
    event_tx: SerialApiEventSender,

    /// The serial API command that's currently being executed
    serial_api_command: Option<SerialApiCommandState>,

    // Some context that's needed for encoding and decoding commands
    storage: Arc<SerialApiStorage>,
    callback_id: WrappingCounter<u8>,
}

pub struct SerialApiAdapter {
    pub serial_in: SerialFrameSender,
    pub serial_out: SerialFrameReceiver,
    pub logs: LogReceiver,
    pub input_tx: SerialApiInputSender,
    pub event_rx: SerialApiEventReceiver,
}

#[derive(Clone)]
pub struct SerialApi {
    input_tx: SerialApiInputSender,
    pub(crate) storage: Arc<SerialApiStorage>,
}

impl SerialApi {
    pub fn new() -> (Self, SerialApiActor, SerialApiAdapter) {
        let (serial_in_tx, serial_in_rx) = mpsc::channel(16);
        let (serial_out_tx, serial_out_rx) = mpsc::channel(16);
        let (log_queue_tx, log_queue_rx) = mpsc::channel(16);
        let (input_tx, input_rx) = mpsc::channel(16);
        let (event_tx, event_rx) = mpsc::channel(16);

        let storage = Arc::new(SerialApiStorage::new(NodeIdType::NodeId8Bit));

        let adapter = SerialApiAdapter {
            serial_in: serial_in_tx,
            serial_out: serial_out_rx,
            logs: log_queue_rx,
            input_tx: input_tx.clone(),
            event_rx,
        };

        let handle = SerialApi {
            input_tx: input_tx.clone(),
            storage: storage.clone(),
        };

        let actor = SerialApiActor {
            serial_in: serial_in_rx,
            serial_out: serial_out_tx,
            log_queue: log_queue_tx,
            input_tx,
            input_rx,
            event_tx,
            serial_api_command: None,
            storage,
            callback_id: WrappingCounter::new(),
        };

        (handle, actor, adapter)
    }
}

pub enum SerialApiInput {
    /// Transmit the given frame
    Transmit {
        frame: SerialFrame,
    },
    /// Notify the application that a frame was received
    Receive {
        frame: SerialFrame,
    },
    /// Execute the given command and return the result once it's done
    ExecCommand {
        command: Box<dyn ExecutableCommand>,
        callback: oneshot::Sender<Result<SerialApiMachineResult>>,
    },
    /// Log the given message
    Log {
        log: LogInfo,
        level: Loglevel,
    },
}

pub enum SerialApiEvent {
    /// A command was received that does not belong to the currently executed command
    Unsolicited { command: Command },
}
