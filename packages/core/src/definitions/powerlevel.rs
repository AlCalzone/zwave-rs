use std::fmt::Display;

use custom_debug_derive::Debug;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Powerlevel {
    #[debug(format = "{:.1} dBm")]
    pub tx_power: f32,
    #[debug(format = "{:.1} dBm")]
    pub measured_at_0_dbm: f32,
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
