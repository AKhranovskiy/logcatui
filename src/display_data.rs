use num_traits::AsPrimitive;
use tui::text::{Span, Spans, Text};
use tui::widgets::Cell;
use tui::widgets::Row;
use unicode_width::UnicodeWidthStr;

use crate::logentry::LogEntry;
use crate::search::matches::{MatchedLine, Matches};
use crate::styles::STYLE_SEARCH_HIGHLIGHT;
use crate::text_utils::create_text;
use crate::text_utils::split_string_at_indices;
use crate::COLUMN_NUMBER;

pub struct DisplayData {
    pub(crate) texts: Vec<String>,
    pub(crate) widths: Vec<usize>,
    pub(crate) wrapped: bool,
}

impl<'a> DisplayData {
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
            .map(|s| UnicodeWidthStr::width(s.as_str()))
            .collect();

        DisplayData {
            texts,
            widths,
            wrapped: false,
        }
    }

    pub fn as_row(
        &self,
        column_offset: usize,
        available_message_width: usize,
        search_results: Option<&MatchedLine>,
    ) -> (Row, usize) {
        if self.wrapped && self.widths.last().unwrap() > &available_message_width {
            let message = self.texts.last().unwrap();
            let text = create_text(message, available_message_width);
            let height = text.height();

            (
                Row::new(
                    self.texts
                        .iter()
                        .take(COLUMN_NUMBER - 1)
                        .skip(column_offset)
                        .map(|c| Cell::from(c.as_str()))
                        .chain(std::iter::once(Cell::from(text))),
                )
                .height(height.as_()),
                height,
            )
        } else {
            (
                Row::new(
                    self.texts
                        .iter()
                        .enumerate()
                        .skip(column_offset)
                        .map(|(index, text)| {
                            search_results
                                .map(MatchedLine::items)
                                .and_then(|columns| columns.exact(index))
                                .map_or_else(
                                    || Cell::from(text.as_str()),
                                    |column| {
                                        let spans = split_string_at_indices(
                                            text,
                                            &column
                                                .iter()
                                                .flat_map(|&p| [p.0, p.1])
                                                .filter(|pos| pos < &text.len())
                                                .collect::<Vec<_>>(),
                                        )
                                        .chunks(2)
                                        .flat_map(|chunks| {
                                            [
                                                chunks.get(0).map_or_else(
                                                    || Span::raw(""),
                                                    |&t| Span::raw(t),
                                                ),
                                                chunks.get(1).map_or_else(
                                                    || Span::raw(""),
                                                    |&t| Span::styled(t, *STYLE_SEARCH_HIGHLIGHT),
                                                ),
                                            ]
                                        })
                                        .collect::<Vec<_>>();
                                        Cell::from(Text::from(Spans(spans)))
                                    },
                                )
                        }),
                ),
                1,
            )
        }
    }
}
