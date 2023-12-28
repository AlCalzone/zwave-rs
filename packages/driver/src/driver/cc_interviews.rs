use proc_macros::impl_cc_interviews;
use zwave_core::definitions::*;
use crate::{Driver, Endpoint, Ready};

pub struct CCInterviewContext<'a> {
    pub driver: &'a Driver<Ready>,
    pub endpoint: &'a dyn Endpoint,
}

// Auto-generate CC interview method and dispatching
// Changes to the #[interview(...)] attributes in the interview implementations
// require proc-macro recompilation or changes to this file in order to be picked up.
impl_cc_interviews!("src/driver/cc_interviews");
// Outputs:
// pub async fn interview_cc(cc: CommandClasses, ctx: &CCInterviewContext<'_>) {
    // ... implementation
// }
