use crate::{Direction, FormattedString, LogFormatter, LogInfo, WithColor};
use termcolor::{Color, ColorSpec};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct DefaultFormatter {
    cs_default: ColorSpec,
    cs_timestamp: ColorSpec,
    cs_label: ColorSpec,
    cs_direction: ColorSpec,
    cs_secondary_tags: ColorSpec,

    line_width: usize,
}

impl DefaultFormatter {
    pub fn new() -> Self {
        let mut cs_timestamp = ColorSpec::default();
        cs_timestamp.set_dimmed(true);

        let mut cs_label = ColorSpec::default();
        cs_label.set_intense(true);
        cs_label.set_bg(Some(Color::Black));

        let cs_direction = cs_timestamp.clone();

        let cs_secondary_tags = cs_timestamp.clone();

        Self {
            cs_default: ColorSpec::default(),
            cs_timestamp,
            cs_label,
            cs_direction,
            cs_secondary_tags,
            line_width: 120,
        }
    }
}

fn get_primary_tag_color_specs(
    highlight_color: Color,
    text_color: Color,
) -> (ColorSpec, ColorSpec) {
    let mut cs_text = ColorSpec::default();
    cs_text.set_fg(Some(text_color));
    cs_text.set_bg(Some(highlight_color));

    let mut cs_delim = ColorSpec::default();
    cs_delim.set_fg(Some(highlight_color));
    cs_delim.set_bg(Some(highlight_color));

    (cs_text, cs_delim)
}

fn str_width(string: &str) -> usize {
    string.graphemes(true).count()
}

impl LogFormatter for DefaultFormatter {
    fn format_log(&self, log: &LogInfo) -> Vec<FormattedString> {
        let timestamp = log
            .timestamp
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        let direction = match log.direction {
            Direction::None => " ",
            Direction::Inbound => "«",
            Direction::Outbound => "»",
        };

        // calculate the width as signed numbers to prevent overflow panics
        let preamble_width =
            (str_width(&timestamp) + 1 + str_width(log.label) + 1 + str_width(direction) + 1)
                as isize;

        let mut ret = vec![
            timestamp.clone().with_color(self.cs_timestamp.clone()),
            " ".into(),
            log.label.with_color(self.cs_label.clone()),
            " ".with_color(self.cs_default.clone()),
            direction.with_color(self.cs_direction.clone()),
            " ".into(),
        ];

        let mut primary_tags_width = 0isize;
        if let Some(primary_tags) = &log.primary_tags {
            // FIXME: Change color based on scope
            let (cs_text, cs_delim) = get_primary_tag_color_specs(Color::Green, Color::Black);

            for tag in primary_tags.iter() {
                ret.push("[".with_color(cs_delim.clone()));
                ret.push(tag.clone().with_color(cs_text.clone()));
                ret.push("]".with_color(cs_delim.clone()));
                ret.push(" ".with_color(self.cs_default.clone()));
                primary_tags_width += (str_width(tag) + 3) as isize; // [ ] and space
            }
        }

        let mut secondary_tag_width = 0isize;
        if let Some(secondary_tag) = &log.secondary_tag {
            secondary_tag_width = (str_width(secondary_tag) + 3) as isize; // ( ) and space
        }

        let available_width = self.line_width as isize - preamble_width;
        let mut last_line_remaining_width = available_width;

        if let Some(lines) = &log.payload.message_lines {
            let num_lines = lines.len();
            for (i, line) in lines.iter().enumerate() {
                let mut is_first = i == 0;
                let is_last = i == num_lines - 1;

                // FIXME: Wrap lines if too long
                let mut graphemes = line.graphemes(true).peekable();
                while graphemes.peek().is_some() {
                    let available_width = available_width
                        - if is_first { primary_tags_width } else { 0isize }
                        - if is_last { secondary_tag_width } else { 0isize };

                    let cur_line: String =
                        graphemes.by_ref().take(available_width as usize).collect();

                    if !is_first {
                        ret.push("\n".into());
                        ret.push(" ".repeat(preamble_width as usize).into());
                    }
                    is_first = false;

                    if is_last && graphemes.peek().is_none() {
                        // If we are in the last input line and have exhausted all graphemes, we
                        // know that the log message was complete. The remaining space can be used
                        // for the secondary tag.
                        last_line_remaining_width = available_width - str_width(&cur_line) as isize;
                    }

                    ret.push(cur_line.with_color(self.cs_default.clone()));
                }
            }
        } else {
            last_line_remaining_width -= primary_tags_width + secondary_tag_width;
        }

        if let Some(secondary_tag) = &log.secondary_tag {
            let padding = last_line_remaining_width;

            if padding > 0 {
                // The tag fits in the remaining space
                ret.push(" ".repeat(padding as usize).into());
            } else {
                // The tag has to go on a new line
                ret.push("\n".with_color(self.cs_default.clone()));
                let padding = self.line_width as isize - secondary_tag_width;
                if padding > 0 {
                    ret.push(" ".repeat(padding as usize).into());
                } else {
                    // The tag is too long for a single line, just print it anyways
                }
            }
            ret.push(format!(" ({})", secondary_tag).with_color(self.cs_secondary_tags.clone()));
        }

        ret.push("\n".with_color(self.cs_default.clone()));

        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::LogPayload;
    use zwave_serial::frame::ControlFlow;

    #[test]
    fn test() {
        let fmt = DefaultFormatter::new();

        // Lines with secondary tags should have the same length
        let log = LogInfo::builder()
            .label("SERIAL")
            .direction(Direction::Outbound)
            .primary_tags(vec![ControlFlow::ACK.to_string().into()])
            .secondary_tag("0x06".into())
            .payload(LogPayload {
                message_lines: None,
                payload: None,
            })
            .build();
        let formatted = fmt.format_log(&log);
        let formatted1 = formatted
            .iter()
            .map(|f| f.string.clone())
            .collect::<String>();

        let log = LogInfo::builder()
            .label("SERIAL")
            .direction(Direction::Outbound)
            .secondary_tag("7 bytes".into())
            .payload(LogPayload {
                message_lines: Some(vec!["0x01020304050607".into()]),
                payload: None,
            })
            .build();
        let formatted = fmt.format_log(&log);
        let formatted2 = formatted
            .iter()
            .map(|f| f.string.clone())
            .collect::<String>();

        // The actual lines should be 120 chars, but the strings include the final line break
        assert_eq!(str_width(&formatted1), 121);
        assert_eq!(str_width(&formatted2), 121);
    }
}
