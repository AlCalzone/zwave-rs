/// The driver can be in one of multiple states, each of which has a different set of capabilities.
pub trait DriverState {}

/// The driver isn't fully initialized yet
pub struct Init;
impl DriverState for Init {}

/// The driver is ready to use normally
pub struct Ready;
impl DriverState for Ready {}
