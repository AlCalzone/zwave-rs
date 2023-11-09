use crate::{parse::empty, prelude::*};

pub struct GetSerialApiInitDataRequest {}

impl GetSerialApiInitDataRequest {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parsable for GetSerialApiInitDataRequest {
    fn parse(i: parse::Input) -> parse::Result<Self> {
        // No payload
        Ok((i, Self {}))
    }
}

impl Serializable for GetSerialApiInitDataRequest {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        empty()
    }
}

impl CommandRequest for GetSerialApiInitDataRequest {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &Command) -> bool {
        response.command_type() == CommandType::Response
            && response.function_type() == self.function_type()
    }

    fn expects_callback(&self) -> bool {
        false
    }

    fn test_callback(&self, _callback: &Command) -> bool {
        false
    }

    fn callback_id(&self) -> Option<u8> {
        return None;
    }

    fn set_callback_id(&mut self, _callback_id: Option<u8>) {
        // This command doesn't use a callback ID
    }
}

pub struct GetSerialApiInitDataResponse {}

impl Parsable for GetSerialApiInitDataResponse {
    fn parse(i: parse::Input) -> parse::Result<Self> {
        todo!()
    }
}

impl Serializable for GetSerialApiInitDataResponse {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        move |out| todo!()
    }
}