use crate::{ControllerCommandError, Driver, Endpoint, ExecNodeCommandError, Node, Ready};
use proc_macros::impl_cc_apis;
use thiserror::Error;
use zwave_core::definitions::*;

pub struct CCInterviewContext<'a> {
    pub driver: &'a Driver<Ready>,
    pub endpoint: &'a dyn Endpoint<'a>,
}

pub trait CCAPI<'a> {
    fn new(endpoint: &'a dyn Endpoint<'a>) -> Self
    where
        Self: Sized;

    fn cc_id(&self) -> CommandClasses;
    fn cc_version(&self) -> u8;

    fn interview_depends_on(&self) -> &'static [CommandClasses] {
        &[]
    }

    #[allow(async_fn_in_trait)]
    async fn interview(&self, ctx: &CCInterviewContext<'_>) -> CCAPIResult<()>;

    #[allow(async_fn_in_trait)]
    async fn refresh_values(&self) -> CCAPIResult<()>;
}

// Auto-generate CC APIs and dispatching interview methods
// Changes to the trait implementations require proc-macro recompilation or changes to this file in order to be picked up.
impl_cc_apis!("src/node/cc_api");

impl<'a> Node<'a> {
    pub fn cc_api(&self) -> CCAPIs {
        CCAPIs::new(self)
    }
}

/// The result of a CC API call
pub type CCAPIResult<T> = Result<T, CCAPIError>;

#[derive(Error, Debug)]
/// Defines the possible errors for a CC API call
pub enum CCAPIError {
    #[error("Controller command error: {0}")]
    Controller(ControllerCommandError),
    #[error("The node did not acknowledge the command")]
    NodeNoAck,
}

impl From<ExecNodeCommandError> for CCAPIError {
    fn from(err: ExecNodeCommandError) -> Self {
        match err {
            ExecNodeCommandError::Controller(err) => Self::Controller(err),
            ExecNodeCommandError::NodeNoAck => Self::NodeNoAck,
            ExecNodeCommandError::NodeTimeout => {
                panic!("Timed out CC API call should have been converted to None")
            }
        }
    }
}

macro_rules! expect_cc_or_timeout {
    ($actual:expr, $expected:ident) => {
        match $actual {
            Ok(Some(zwave_cc::commandclass::CC::$expected(result))) => Some(result),
            Ok(_) => {
                // If we receive a different CC than expected,
                // there's a bug in the CC implementation
                panic!("expected {}", stringify!($expected));
            }
            Err($crate::ExecNodeCommandError::NodeTimeout) => {
                // In the CC API, timeouts are translated to no response
                None
            }
            Err(e) => return Err(e.into()),
        }
    };
}
pub(crate) use expect_cc_or_timeout;
