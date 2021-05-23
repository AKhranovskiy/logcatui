use tui::widgets::Cell;
use tui::widgets::Row;
use unicode_width::UnicodeWidthStr;

use crate::logentry::LogEntry;
use crate::text_utils::create_text;
use crate::COLUMN_NUMBER;

#[allow(dead_code)]
pub struct DisplayData<'a> {
    log_entry: &'a LogEntry,
    pub(crate) texts: Vec<String>,
    pub(crate) widths: Vec<u16>,
    pub(crate) wrapped: bool,
}

impl<'a> DisplayData<'a> {
    pub(crate) fn new(entry: &'a LogEntry) -> Self {
        let texts = vec![
            entry.timestamp.format("%F %H:%M:%S%.3f").to_string(),
            entry.process_id.to_string(),
            entry.thread_id.to_string(),
            entry.log_level.to_string(),
            entry.tag.to_string(),
            entry.message.to_string(),
        ];
        assert_eq!(texts.len(), COLUMN_NUMBER);

        let widths = texts
            .iter()
            .map(|s| UnicodeWidthStr::width(s.as_str()) as u16)
            .collect();

        DisplayData {
            log_entry: entry,
            texts,
            widths,
            wrapped: false,
        }
    }
    pub fn as_row(&self, column_offset: usize, available_message_width: u16) -> Row {
        if self.wrapped && self.widths.last().unwrap() > &available_message_width {
            let message = self.texts.last().unwrap();
            let text = create_text(message, available_message_width);
            let height = text.height() as u16;

            Row::new(
                self.texts
                    .iter()
                    .take(COLUMN_NUMBER - 1)
                    .skip(column_offset)
                    .map(|c| Cell::from(c.as_str()))
                    .chain(std::iter::once(Cell::from(text))),
            )
            .height(height)
        } else {
            Row::new(
                self.texts
                    .iter()
                    .skip(column_offset)
                    .map(|t| Cell::from(t.as_str())),
            )
        }
    }
}
