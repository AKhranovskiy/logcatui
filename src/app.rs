use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction::Vertical, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use tui::Frame;

use crate::log_table::LogTable;
use crate::logentry::LogEntry;
use crate::COLUMN_HEADERS;

lazy_static! {
    static ref STYLE_HEADER: Style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(Color::White)
        .bg(Color::DarkGray);
    static ref STYLE_SELECTED_ROW: Style = Style::default().add_modifier(Modifier::REVERSED);
}

pub struct App<'a> {
    pub should_quit: bool,
    title: String,
    table: LogTable<'a>,
    fps: fps_counter::FPSCounter,
    input_event_message: String,
}

impl<'a> App<'a> {
    pub fn init(title: String, model: &'a [LogEntry]) -> Self {
        let mut table = LogTable::new(&model);
        // Select first line by default;
        table.next();

        App {
            title,
            table,
            fps: fps_counter::FPSCounter::new(),
            should_quit: false,
            input_event_message: String::new(),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(f.size());

        self.table.viewport = Rect::new(0, 0, f.size().width, f.size().height - 4_u16);

        let instant = std::time::Instant::now();

        let header_cells = COLUMN_HEADERS
            .iter()
            .skip(self.table.column_offset)
            .map(|h| Cell::from(*h));

        let header = Row::new(header_cells).style(*STYLE_HEADER);

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
            .highlight_style(*STYLE_SELECTED_ROW)
            .column_spacing(1)
            .widths(&constraints);

        let table_built = instant.elapsed();

        f.render_stateful_widget(t, chunks[0], &mut self.table.state);

        let table_rendered = instant.elapsed();

        let bottom_block = Paragraph::new(format!(
            "Row {}/{} FPS: {} table built in {}ms, table rendered in {}ms, {}",
            self.table.state.selected().map_or(0, |v| v + 1),
            self.table.len(),
            self.fps.tick(),
            table_built.as_millis(),
            (table_rendered - table_built).as_millis(),
            self.input_event_message
        ))
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left);
        f.render_widget(bottom_block, chunks[1]);
    }

    fn copy_line(&self) {
        if let Some(selected) = self.table.state.selected() {
            if let Some(entry) = self.table.model.get(selected) {
                ClipboardProvider::new()
                    .map(|mut ctx: ClipboardContext| ctx.set_contents(format!("{}", entry)))
                    .flatten()
                    .map_err(|e| dbg!(e))
                    .unwrap();
            }
        }
    }

    fn copy_message(&self) {
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

    fn quit(&mut self) {
        self.should_quit = true
    }

    pub fn input(&mut self, event: &KeyEvent) {
        self.input_event_message.clear();

        match event.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c') => {
                if with_ctrl(event) {
                    self.quit()
                }
            }

            KeyCode::Down => {
                self.table.next();
            }
            KeyCode::Up => {
                self.table.previous();
            }
            KeyCode::PageDown => {
                self.table.next_page();
            }
            KeyCode::PageUp => {
                self.table.previous_page();
            }
            KeyCode::Left => {
                self.table.left();
            }
            KeyCode::Right => {
                self.table.right();
            }
            KeyCode::Enter => {
                self.table.wrap_message();
            }
            KeyCode::Char('y') => {
                self.copy_line();
                self.input_event_message = format!(
                    "Copied the line {} to clipboard",
                    self.table.state.selected().unwrap()
                );
            }
            KeyCode::Char('Y') => {
                self.copy_message();
                self.input_event_message = format!(
                    "Copied the message from the line {} to clipboard",
                    self.table.state.selected().unwrap_or(0) + 1
                );
            }
            KeyCode::Home => self.table.first(),
            KeyCode::End => self.table.last(),
            _ => {}
        }
    }
}

fn with_ctrl(event: &KeyEvent) -> bool {
    event.modifiers.contains(KeyModifiers::CONTROL)
}
