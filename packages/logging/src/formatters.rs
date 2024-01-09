use crate::{LogFormatter, LogInfo, FormattedString};

pub struct DefaultFormatter {}

impl DefaultFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl LogFormatter for DefaultFormatter {
    fn format_log(&self, log: &LogInfo) -> Vec<FormattedString> {
        // TODO: implement this!
        vec![format!("log {:?}", log.payload).into()]
    }
}
