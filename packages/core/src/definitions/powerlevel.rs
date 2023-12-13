use custom_debug_derive::Debug;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Powerlevel {
    #[debug(format = "{:.1} dBm")]
    pub tx_power: f32,
    #[debug(format = "{:.1} dBm")]
    pub measured_at_0_dbm: f32,
}
