use crate::{LogFormatter, LogInfo, Logger};
use termcolor::WriteColor;
use zwave_core::log::Loglevel;

pub struct BaseLogger {
    pub level: Loglevel,
    pub writer: Box<dyn WriteColor>,
    pub formatter: Box<dyn LogFormatter>,
}

impl Logger for BaseLogger {
    fn log(&mut self, log: LogInfo, level: Loglevel) {
        if level > self.level {
            return;
        }
        let formatted = self.formatter.format_log(&log, level);
        for str in formatted {
            if let Some(color) = str.color {
                let _ = self.writer.set_color(&color);
            }
            let _ = self.writer.write_all(str.string.as_bytes());
        }
    }

    fn log_level(&self) -> Loglevel {
        self.level
    }

    fn set_log_level(&mut self, level: Loglevel) {
        self.level = level;
    }
}
