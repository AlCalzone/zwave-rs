#![allow(clippy::new_without_default)]

use crate::prelude::*;
use zwave_core::prelude::*;

use zwave_core::encoding::{self, encoders::empty};

#[derive(Debug, Clone, PartialEq)]
pub struct SoftResetRequest {}

impl SoftResetRequest {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parsable for SoftResetRequest {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl Serializable for SoftResetRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        // No payload
        empty()
    }
}

impl CommandRequest for SoftResetRequest {
    fn expects_response(&self) -> bool {
        false
    }

    fn expects_callback(&self) -> bool {
        false
    }
}
