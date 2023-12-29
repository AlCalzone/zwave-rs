use crate::{Driver, Endpoint, Ready};
use proc_macros::impl_cc_apis;
use zwave_core::definitions::*;

pub struct CCInterviewContext<'a> {
    pub driver: &'a Driver<Ready>,
    pub endpoint: &'a dyn Endpoint,
}

pub trait CCAPI<'a> {
    fn new(endpoint: &'a dyn Endpoint) -> Self
    where
        Self: Sized;

    fn cc_id(&self) -> CommandClasses;
    fn cc_version(&self) -> u8;

    #[allow(async_fn_in_trait)]
    async fn interview(&self, ctx: &CCInterviewContext<'_>);
}

// Auto-generate CC APIs and dispatching interview methods
// Changes to the trait implementations require proc-macro recompilation or changes to this file in order to be picked up.
impl_cc_apis!("src/driver/cc_api");
