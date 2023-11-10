use crate::{parse::empty, prelude::*};

#[derive(Debug, Clone, PartialEq)]
pub struct SoftResetRequest {}

impl SoftResetRequest {
	pub fn new() -> Self {
		Self {}
	}
}

impl Parsable for SoftResetRequest {
	fn parse(i: parse::Input) -> parse::Result<Self> {
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

	fn test_response(&self, _response: &Command) -> bool {
		false
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
		// No callback
	}
}