use std::fmt::Display;

use crate::encoding::{self, Parsable, Serializable};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::*;
use nom::{combinator::map, error::context, number::complete::be_u8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum SerialAPIWakeUpReason {
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

impl Display for SerialAPIWakeUpReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerialAPIWakeUpReason::Reset => write!(f, "Reset"),
            SerialAPIWakeUpReason::Timer => write!(f, "Timer"),
            SerialAPIWakeUpReason::WakeUpBeam => write!(f, "Wake up beam"),
            SerialAPIWakeUpReason::WatchdogReset => write!(f, "Reset by watchdog"),
            SerialAPIWakeUpReason::ExternalInterrupt => write!(f, "External interrupt"),
            SerialAPIWakeUpReason::PowerUp => write!(f, "Powered up"),
            SerialAPIWakeUpReason::USBSuspend => write!(f, "USB suspend"),
            SerialAPIWakeUpReason::SoftwareReset => write!(f, "Reset by software"),
            SerialAPIWakeUpReason::EmergencyWatchdogReset => write!(f, "Emergency watchdog reset"),
            SerialAPIWakeUpReason::BrownoutCircuit => write!(f, "Reset by brownout circuit"),
            SerialAPIWakeUpReason::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Parsable for SerialAPIWakeUpReason {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        context(
            "SerialAPIWakeUpReason",
            map(be_u8, |x| SerialAPIWakeUpReason::try_from(x).unwrap()),
        )(i)
    }
}

impl Serializable for SerialAPIWakeUpReason {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        cf::bytes::be_u8(*self as u8)
    }
}
