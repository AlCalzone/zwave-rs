use crate::munch::{
    bytes::be_u8,
    combinators::{context, map_res},
};
use crate::prelude::*;
use bytes::Bytes;
use cookie_factory as cf;
use custom_debug_derive::Debug;
use proc_macros::TryFromRepr;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum SerialApiWakeUpReason {
    /// The Z-Wave API Module has been woken up by reset or external interrupt.
    Reset = 0x00,
    /// The Z-Wave API Module has been woken up by a timer.
    Timer = 0x01,
    /// The Z-Wave API Module has been woken up by a Wake Up Beam.
    WakeUpBeam = 0x02,
    /// The Z-Wave API Module has been woken up by a reset triggered by the watchdog.
    WatchdogReset = 0x03,
    /// The Z-Wave API Module has been woken up by an external interrupt.
    ExternalInterrupt = 0x04,
    /// The Z-Wave API Module has been woken up by a powering up.
    PowerUp = 0x05,
    /// The Z-Wave API Module has been woken up by USB Suspend.
    USBSuspend = 0x06,
    /// The Z-Wave API Module has been woken up by a reset triggered by software.
    SoftwareReset = 0x07,
    /// The Z-Wave API Module has been woken up by an emergency watchdog reset.
    EmergencyWatchdogReset = 0x08,
    /// The Z-Wave API Module has been woken up by a reset triggered by brownout circuit.
    BrownoutCircuit = 0x09,
    /// The Z-Wave API Module has been woken up by an unknown reason.
    Unknown = 0xff,
}

impl Display for SerialApiWakeUpReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerialApiWakeUpReason::Reset => write!(f, "Reset"),
            SerialApiWakeUpReason::Timer => write!(f, "Timer"),
            SerialApiWakeUpReason::WakeUpBeam => write!(f, "Wake up beam"),
            SerialApiWakeUpReason::WatchdogReset => write!(f, "Reset by watchdog"),
            SerialApiWakeUpReason::ExternalInterrupt => write!(f, "External interrupt"),
            SerialApiWakeUpReason::PowerUp => write!(f, "Powered up"),
            SerialApiWakeUpReason::USBSuspend => write!(f, "USB suspend"),
            SerialApiWakeUpReason::SoftwareReset => write!(f, "Reset by software"),
            SerialApiWakeUpReason::EmergencyWatchdogReset => write!(f, "Emergency watchdog reset"),
            SerialApiWakeUpReason::BrownoutCircuit => write!(f, "Reset by brownout circuit"),
            SerialApiWakeUpReason::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Parsable for SerialApiWakeUpReason {
    fn parse(i: &mut Bytes) -> crate::munch::ParseResult<Self> {
        context(
            "SerialApiWakeUpReason",
            map_res(be_u8, SerialApiWakeUpReason::try_from),
        )
        .parse(i)
    }
}

impl Serializable for SerialApiWakeUpReason {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
