#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use zwave_core::submodule;

submodule!(driver);
pub mod error;
submodule!(controller);
submodule!(node);
submodule!(serial_api);

pub type LogSender =
    zwave_pal::channel::Sender<(zwave_logging::LogInfo, zwave_core::log::Loglevel)>;
pub type LogReceiver =
    zwave_pal::channel::Receiver<(zwave_logging::LogInfo, zwave_core::log::Loglevel)>;
