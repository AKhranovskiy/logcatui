use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use clipboard::{ClipboardContext, ClipboardProvider};
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::Direction::Vertical;
use tui::layout::{Alignment, Constraint, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use tui::Frame;

use crate::log_table::LogTable;
use crate::logentry::LogEntry;
use crate::COLUMN_HEADERS;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum StyleKey {
    Header,
    SelectedRow,
}

impl Hash for StyleKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(*self as u32)
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        data.iter().for_each(|&s| state.write_u32(s as u32))
    }
}

pub struct App<'a> {
    pub should_quit: bool,
    title: String,
    styles: HashMap<StyleKey, Style>,
    table: LogTable<'a>,
    fps: fps_counter::FPSCounter,
    last_key: Option<Key>,
}

fn init_styles() -> HashMap<StyleKey, Style> {
    let mut styles = HashMap::new();
    styles.insert(
        StyleKey::Header,
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::White)
            .bg(Color::DarkGray),
    );
    styles.insert(
        StyleKey::SelectedRow,
        Style::default().add_modifier(Modifier::REVERSED),
    );
    styles
}

impl<'a> App<'a> {
    pub fn init(title: String, model: &'a [LogEntry]) -> Self {
        let mut table = LogTable::new(&model);
        // Select first line by default;
        table.next();

        App {
            title,
            styles: init_styles(),
            table,
            fps: fps_counter::FPSCounter::new(),
            should_quit: false,
            last_key: None,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(f.size());

        self.table.viewport = Rect::new(0, 0, f.size().width, f.size().height - 4u16);

        let instant = std::time::Instant::now();

        let header_cells = COLUMN_HEADERS
            .iter()
            .skip(self.table.column_offset)
            .map(|h| Cell::from(*h));

        let header = Row::new(header_cells).style(self.styles[&StyleKey::Header]);

        let available_message_width = self.table.available_message_width();
        let rows = self
            .table
            .display_data
            .iter()
            .map(|data| data.as_row(self.table.column_offset, available_message_width));

        let constraints = self.table.column_constraints();
        let t = Table::new(rows)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title.as_str()),
            )
            .highlight_style(self.styles[&StyleKey::SelectedRow])
            .column_spacing(1)
            .widths(&constraints);

        let table_built = instant.elapsed();

        f.render_stateful_widget(t, chunks[0], &mut self.table.state);

        let table_rendered = instant.elapsed();

        let bottom_block = Paragraph::new(format!(
            "Row {}/{} FPS: {} table built in {}ms, table rendered in {}ms, last key: {:?}",
            self.table.state.selected().map(|v| v + 1).unwrap_or(0),
            self.table.len(),
            self.fps.tick(),
            table_built.as_millis(),
            (table_rendered - table_built).as_millis(),
            self.last_key
        ))
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left);
        f.render_widget(bottom_block, chunks[1]);
    }

    fn copy_line(&self) {
        if let Some(selected) = self.table.state.selected() {
            if let Some(entry) = self.table.model.get(selected) {
                ClipboardProvider::new()
                    .map(|mut ctx: ClipboardContext| ctx.set_contents(entry.message.clone()))
                    .flatten()
                    .map_err(|e| dbg!(e))
                    .unwrap();
            }
        }
    }

    pub fn input(&mut self, key: &Key) {
        self.last_key = Some(*key);
        match key {
            Key::Char('q') | Key::Ctrl('c') => self.should_quit = true,
            Key::Down => {
                self.table.next();
            }
            Key::Up => {
                self.table.previous();
            }
            Key::PageDown => {
                self.table.next_page();
            }
            Key::PageUp => {
                self.table.previous_page();
            }
            Key::Left => {
                self.table.left();
            }
            Key::Right => {
                self.table.right();
            }
            Key::Char('\n') => {
                self.table.wrap_message();
            }
            Key::Char('y') => {
                self.copy_line();
            }
            Key::Home => self.table.first(),
            Key::End => self.table.last(),
            _ => {}
        }
    }
}
