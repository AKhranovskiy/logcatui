use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use num_traits::AsPrimitive;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction::Vertical, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use tui::Frame;
use unicode_width::UnicodeWidthStr;

use crate::log_table::LogTable;
use crate::logentry::LogEntry;
use crate::COLUMN_HEADERS;

lazy_static! {
    static ref STYLE_HEADER: Style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(Color::White)
        .bg(Color::DarkGray);
    static ref STYLE_SELECTED_ROW: Style = Style::default().add_modifier(Modifier::REVERSED);
    static ref STYLE_QUICK_SEARCH: Style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
}

pub struct App<'a> {
    pub should_quit: bool,
    title: String,
    table: LogTable<'a>,
    fps: fps_counter::FPSCounter,
    input_event_message: String,
    quick_search: QuickSearchState,
}

struct AppLayout {
    table: Rect,
    quick_search: Rect,
    status_bar: Rect,
}

struct QuickSearchState {
    mode: QuickSearchMode,
    input: String,
}

impl Default for QuickSearchState {
    fn default() -> Self {
        Self {
            mode: QuickSearchMode::Off,
            input: String::new(),
        }
    }
}

enum QuickSearchMode {
    Off,
    Input,
    Iteration,
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
            quick_search: QuickSearchState::default(),
        }
    }

    fn layout<B: Backend>(&self, f: &mut Frame<B>) -> AppLayout {
        let quick_search_height: u16 = match self.quick_search.mode {
            QuickSearchMode::Off => 0,
            QuickSearchMode::Input | QuickSearchMode::Iteration => 1,
        };

        let chunks = Layout::default()
            .direction(Vertical)
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Length(quick_search_height),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(f.size());

        AppLayout {
            table: chunks[0],
            quick_search: chunks[1],
            status_bar: chunks[2],
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let layout = self.layout(f);

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

        f.render_stateful_widget(t, layout.table, &mut self.table.state);

        let table_rendered = instant.elapsed();

        match self.quick_search.mode {
            QuickSearchMode::Off => {}
            QuickSearchMode::Input => {
                let block = Paragraph::new(format!("/ {}", self.quick_search.input))
                    .block(Block::default().borders(Borders::NONE));
                f.render_widget(block, layout.quick_search);
                let w: u16 = self.quick_search.input.width().as_();
                f.set_cursor(
                    // Put cursor past the end of the input text
                    layout.quick_search.x + 1 + w + 1,
                    layout.quick_search.y,
                )
            }
            QuickSearchMode::Iteration => {
                let block = Paragraph::new(format!("/{}", self.quick_search.input))
                    .style(*STYLE_QUICK_SEARCH)
                    .block(Block::default().borders(Borders::NONE));
                f.render_widget(block, layout.quick_search);
            }
        }

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

        f.render_widget(bottom_block, layout.status_bar);
    }

    fn copy_line(&mut self) {
        if let Some(selected) = self.table.state.selected() {
            if let Some(entry) = self.table.model.get(selected) {
                self.input_event_message = match ClipboardProvider::new()
                    .and_then(|mut ctx: ClipboardContext| ctx.set_contents(format!("{}", entry)))
                {
                    Ok(()) => format!("Copied the line {} to clipboard", selected + 1),
                    Err(ref e) => format!("Failed to copy line {}", e),
                }
            }
        }
    }

    fn copy_message(&mut self) {
        if let Some(selected) = self.table.state.selected() {
            if let Some(entry) = self.table.model.get(selected) {
                self.input_event_message = match ClipboardProvider::new()
                    .and_then(|mut ctx: ClipboardContext| ctx.set_contents(entry.message.clone()))
                {
                    Ok(()) => format!(
                        "Copied the message from the line {} to clipboard",
                        selected + 1
                    ),
                    Err(ref e) => format!("Failed to copy message {}", e),
                }
            }
        }
    }

    fn quit(&mut self) {
        self.should_quit = true
    }

    fn regular_input(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c') => {
                if with_ctrl(event) {
                    self.quit()
                }
            }
            KeyCode::Down => self.table.next(),
            KeyCode::Up => self.table.previous(),
            KeyCode::PageDown => self.table.next_page(),
            KeyCode::PageUp => self.table.previous_page(),
            KeyCode::Left => self.table.left(),
            KeyCode::Right => self.table.right(),
            KeyCode::Enter => self.table.wrap_message(),
            KeyCode::Char('y') => self.copy_line(),
            KeyCode::Char('Y') => self.copy_message(),
            KeyCode::Home => self.table.first(),
            KeyCode::End => self.table.last(),
            KeyCode::Char('/') => self.quick_search.mode = QuickSearchMode::Input,
            _ => {}
        }
    }
    pub fn input(&mut self, event: &KeyEvent) {
        self.input_event_message.clear();

        match self.quick_search.mode {
            QuickSearchMode::Off => self.regular_input(&event),
            QuickSearchMode::Input => match event.code {
                KeyCode::Esc => {
                    self.quick_search.mode = QuickSearchMode::Off;
                    self.quick_search.input.clear();
                }
                KeyCode::Enter => {
                    if self.quick_search.input.is_empty() {
                        self.quick_search.mode = QuickSearchMode::Off
                    } else {
                        self.quick_search.mode = QuickSearchMode::Iteration
                    }
                }
                KeyCode::Backspace => self.quick_search.input.clear(),
                KeyCode::Char(c) => self.quick_search.input.push(c),
                _ => {}
            },
            QuickSearchMode::Iteration => match event.code {
                KeyCode::Esc => self.quick_search.mode = QuickSearchMode::Off,
                KeyCode::Char('n') => {
                    self.input_event_message = "Go to next search result.".to_string();
                }
                KeyCode::Char('N') => {
                    self.input_event_message = "Go to prev search result.".to_string();
                }
                _ => self.regular_input(&event),
            },
        }
    }
}

fn with_ctrl(event: &KeyEvent) -> bool {
    event.modifiers.contains(KeyModifiers::CONTROL)
}
