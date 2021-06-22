use num_traits::AsPrimitive;
use tui::layout::{Constraint, Rect};

use crate::display_data::DisplayData;
use crate::logentry::LogEntry;
use crate::COLUMN_NUMBER;

pub struct LogTable<'a> {
    pub(crate) model: &'a [LogEntry],
    pub(crate) display_data: Vec<DisplayData<'a>>,
    pub(crate) column_widths: Vec<usize>,
    pub(crate) viewport: Rect,
    pub(crate) column_offset: usize,
}

impl<'a> LogTable<'a> {
    pub(crate) fn new(model: &[LogEntry]) -> LogTable {
        let display_data: Vec<DisplayData> =
            model.iter().map(|entry| DisplayData::new(entry)).collect();

        let mut column_widths =
            display_data
                .iter()
                .fold(vec![0_usize; COLUMN_NUMBER], |max_widths, data| {
                    data.widths
                        .iter()
                        .zip(max_widths)
                        .map(|(w, mw)| *w.max(&mw))
                        .collect()
                });

        // Override width of TAG column because the maximum length is almost always too much.
        column_widths[4] = 18;

        LogTable {
            model,
            display_data,
            column_widths,
            viewport: Rect::default(),
            column_offset: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.model.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.model.is_empty()
    }

    pub fn right(&mut self) {
        self.column_offset = self.column_offset.saturating_add(1).min(COLUMN_NUMBER - 1);
    }
    pub fn left(&mut self) {
        self.column_offset = self.column_offset.saturating_sub(1);
    }

    pub fn wrap_message(&mut self, index: usize) {
        if let Some(data) = self.display_data.get_mut(index) {
            data.wrapped = !data.wrapped;
        }
    }

    pub fn available_message_width(&self) -> usize {
        let width_without_message = self
            .column_widths
            .iter()
            .take(COLUMN_NUMBER - 1)
            .skip(self.column_offset)
            .sum::<usize>();
        let column_spacing = COLUMN_NUMBER - self.column_offset;
        self.viewport.width as usize - 1 - width_without_message - column_spacing
    }

    pub(crate) fn column_constraints(&self) -> Vec<Constraint> {
        self.column_widths[self.column_offset..]
            .iter()
            .map(|&w| Constraint::Length(w.as_()))
            .collect::<Vec<_>>()
    }
}
