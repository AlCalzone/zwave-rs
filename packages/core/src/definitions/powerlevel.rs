use std::fmt::{Debug, Display};
#[derive(Default, Clone, Copy, PartialEq)]
pub struct Powerlevel {
    pub tx_power: f32,
    pub measured_at_0_dbm: f32,
}

impl Debug for Powerlevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Powerlevel")
            .field("tx_power", &format_args!("{:.1} dBm", self.tx_power))
            .field(
                "measured_at_0_dbm",
                &format_args!("{:.1} dBm", self.measured_at_0_dbm),
            )
            .finish()
    }
}

impl Display for Powerlevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.1} dBm (measured {:.1} dBm)",
            self.tx_power, self.measured_at_0_dbm
        )
    }
}
