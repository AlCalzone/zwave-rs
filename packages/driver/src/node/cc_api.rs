use crate::{ControllerCommandError, Endpoint, EndpointLike, ExecNodeCommandError, Node};
use proc_macros::impl_cc_apis;
use thiserror::Error;
use zwave_core::definitions::*;

pub trait CCAPI<'a> {
    fn new(endpoint: &'a dyn EndpointLike<'a>) -> Self
    where
        Self: Sized;

    fn cc_id(&self) -> CommandClasses;
    fn cc_version(&self) -> u8;

    fn interview_depends_on(&self) -> &'static [CommandClasses] {
        &[]
    }

    #[allow(async_fn_in_trait)]
    async fn interview(&self) -> CCAPIResult<()>;

    #[allow(async_fn_in_trait)]
    async fn refresh_values(&self) -> CCAPIResult<()>;
}

// Auto-generate CC API modules and interview dispatch from all .rs files in src/node/cc_api.
// If a newly added API file is not picked up immediately, touching this file or recompiling the
// proc-macro crate forces Cargo to rerun the macro.
impl_cc_apis!("src/node/cc_api");

impl<'a> Node<'a> {
    pub fn cc_api(&self) -> CCAPIs<'_> {
        CCAPIs::new(self)
    }
}

impl<'a> Endpoint<'a> {
    pub fn cc_api(&self) -> CCAPIs<'_> {
        CCAPIs::new(self)
    }
}

/// The result of a CC API call
pub type CCAPIResult<T> = Result<T, CCAPIError>;

#[derive(Error, Debug)]
/// Defines the possible errors for a CC API call
pub enum CCAPIError {
    #[error("Node {node_id}, endpoint {endpoint} does not support the API command {api_command}")]
    NotSupported {
        node_id: NodeId,
        endpoint: EndpointIndex,
        api_command: &'static str,
    },
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

/// Used to simplify handling the alternative branches when expecting one of multiple CCs.
/// Like `expect_cc_or_timeout!`, timeouts are converted to `None`, unexpected CCs cause a
/// panic, and other errors are returned from the surrounding function.
macro_rules! handle_unexpected_cc_or_timeout {
    (
        $actual:expr,
        $($expected:ident),+ $(,)?
    ) => {
        match $actual {
            Ok(None) | Err($crate::ExecNodeCommandError::NodeTimeout) => None,
            Ok(Some(_)) => {
                // If we receive a different CC than expected,
                // there's a bug in the CC implementation
                panic!(concat!("expected one of ", $(stringify!($expected), ", "),*));
            }
            Err(e) => return Err(e.into()),
        }
    };
}
pub(crate) use handle_unexpected_cc_or_timeout;

macro_rules! cc_api_assert_supported {
    ($self:ident, $cmd:ident) => {
        paste::paste! {
            match $self.[<supports_ $cmd>]() {
                Some(true) => Ok(()),
                _ => Err(crate::CCAPIError::NotSupported {
                    node_id: $self.endpoint.node_id(),
                    endpoint: $self.endpoint.index(),
                    api_command: stringify!($cmd),
                }),
            }?;
        }
    };
}
pub(crate) use cc_api_assert_supported;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExecNodeCommandError;
    use zwave_cc::commandclass::{
        basic::BasicCCReport, binary_switch::BinarySwitchCCReport,
        security::SecurityCCNonceReport,
    };
    use zwave_cc::prelude::CC;
    use zwave_core::security::S0Nonce;
    use zwave_core::prelude::{BinaryReport, LevelReport};

    fn map_expected_response(
        actual: Result<Option<CC>, ExecNodeCommandError>,
    ) -> CCAPIResult<Option<&'static str>> {
        match actual {
            Ok(Some(CC::BasicCCReport(report))) => Ok(Some(match report.current_value {
                LevelReport::Level(_) => "basic",
                LevelReport::Unknown => "basic-unknown",
            })),
            Ok(Some(CC::BinarySwitchCCReport(_))) => Ok(Some("binary")),
            other => Ok(handle_unexpected_cc_or_timeout!(
                other,
                BasicCCReport,
                BinarySwitchCCReport,
            )),
        }
    }

    fn basic_report() -> CC {
        CC::BasicCCReport(BasicCCReport {
            current_value: LevelReport::Level(1),
            target_value: None,
            duration: None,
        })
    }

    fn binary_switch_report() -> CC {
        CC::BinarySwitchCCReport(BinarySwitchCCReport {
            current_value: BinaryReport::On,
            target_value: None,
            duration: None,
        })
    }

    #[test]
    fn expect_ccs_or_timeout_matches_first_expected_cc() {
        let result = map_expected_response(Ok(Some(basic_report())));

        assert!(matches!(result, Ok(Some("basic"))));
    }

    #[test]
    fn expect_ccs_or_timeout_matches_later_expected_cc() {
        let result = map_expected_response(Ok(Some(binary_switch_report())));

        assert!(matches!(result, Ok(Some("binary"))));
    }

    #[test]
    fn expect_ccs_or_timeout_maps_timeout_to_none() {
        let result = map_expected_response(Err(ExecNodeCommandError::NodeTimeout));

        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn expect_ccs_or_timeout_maps_missing_response_to_none() {
        let result = map_expected_response(Ok(None));

        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn expect_ccs_or_timeout_returns_non_timeout_errors() {
        let result = map_expected_response(Err(ExecNodeCommandError::NodeNoAck));

        assert!(matches!(result, Err(CCAPIError::NodeNoAck)));
    }

    #[test]
    #[should_panic(expected = "expected one of BasicCCReport, BinarySwitchCCReport")]
    fn expect_ccs_or_timeout_panics_on_unexpected_cc() {
        let _ = map_expected_response(Ok(Some(CC::SecurityCCNonceReport(
            SecurityCCNonceReport::builder()
                .nonce(S0Nonce::new(&[0; 8]))
                .build(),
        ))))
        .unwrap();
    }
}
