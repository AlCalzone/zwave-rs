use zwave_core::submodule;

submodule!(driver);
pub mod error;
submodule!(controller);
submodule!(node);
submodule!(serial_api);

type LogSender =
    futures::channel::mpsc::Sender<(zwave_logging::LogInfo, zwave_core::log::Loglevel)>;
type LogReceiver =
    futures::channel::mpsc::Receiver<(zwave_logging::LogInfo, zwave_core::log::Loglevel)>;
