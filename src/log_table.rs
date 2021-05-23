use tui::layout::Rect;
use tui::widgets::TableState;

use crate::display_data::DisplayData;
use crate::logentry::LogEntry;
use crate::COLUMN_NUMBER;

pub struct LogTable<'a> {
    pub(crate) state: TableState,
    model: &'a [LogEntry],
    pub(crate) display_data: Vec<DisplayData<'a>>,
    pub(crate) column_widths: Vec<u16>,
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
                .fold(vec![0u16; COLUMN_NUMBER], |max_widths, data| {
                    data.widths
                        .iter()
                        .zip(max_widths)
                        .map(|(w, mw)| *w.max(&mw))
                        .collect()
                });

        // Override width of TAG column because the maximum length is almost always too much.
        column_widths[4] = 18;

        LogTable {
            state: TableState::default(),
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

    pub fn next(&mut self) {
        let next_item = self
            .state
            .selected()
            .map(|idx| idx.saturating_add(1).min(self.len() - 1))
            .or(Some(0));
        self.state.select(next_item);
    }

    fn page_size(&self) -> usize {
        self.viewport.height as usize
    }

    pub fn next_page(&mut self) {
        let next_item = self
            .state
            .selected()
            .map(|idx| idx.saturating_add(self.page_size()).min(self.len() - 1))
            .or(Some(0));
        self.state.select(next_item);
    }

    pub fn previous(&mut self) {
        let prev_item = self
            .state
            .selected()
            .map(|idx| idx.saturating_sub(1))
            .or(Some(0));
        self.state.select(prev_item);
    }
    pub fn previous_page(&mut self) {
        let prev_item = self
            .state
            .selected()
            .map(|idx| idx.saturating_sub(self.page_size()))
            .or(Some(0));
        self.state.select(prev_item);
    }

    pub fn right(&mut self) {
        self.column_offset = self.column_offset.saturating_add(1).min(COLUMN_NUMBER - 1)
    }
    pub fn left(&mut self) {
        self.column_offset = self.column_offset.saturating_sub(1)
    }

    pub fn wrap_message(&mut self) {
        if let Some(selected) = self.state.selected() {
            if let Some(data) = self.display_data.get_mut(selected) {
                data.wrapped = !data.wrapped
            }
        }
    }

    pub fn available_message_width(&self) -> u16 {
        let width_without_message = self
            .column_widths
            .iter()
            .take(COLUMN_NUMBER - 1)
            .skip(self.column_offset)
            .sum::<u16>();
        let column_spacing: u16 = (COLUMN_NUMBER - self.column_offset) as u16;
        self.viewport.width - width_without_message - column_spacing
    }

    // pub fn
}
