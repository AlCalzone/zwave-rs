use zwave_core::{definitions::CommandClasses, submodule};

use crate::{Driver, Endpoint, Ready};

// TODO: Generate this
submodule!(basic);

pub struct CCInterviewContext<'a> {
    pub driver: &'a Driver<Ready>,
    pub endpoint: &'a dyn Endpoint,
}

pub async fn interview_cc(cc: CommandClasses, ctx: &CCInterviewContext<'_>) {
    // TODO: Generate this
    match cc {
        CommandClasses::Basic => interview_basic_cc(ctx).await,
        _ => {
            // No interview procedure
        },
    }
}
