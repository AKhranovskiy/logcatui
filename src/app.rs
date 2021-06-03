use std::collections::BTreeSet;

use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use num_traits::AsPrimitive;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction::Vertical, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
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
    state: TableState,
    fps: fps_counter::FPSCounter,
    input_event_message: String,
    quick_search: QuickSearchState,
    vertical_offset: usize,
}

struct AppLayout {
    table: Rect,
    quick_search: Rect,
    status_bar: Rect,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct QueryMatch {
    index: usize,
    column: usize, // todo use enum constant
                   // todo matched positions
}

impl QueryMatch {
    fn new(index: usize, column: usize) -> Self {
        Self { index, column }
    }
}

struct QuickSearchState {
    mode: QuickSearchMode,
    input: String,
    matches: BTreeSet<QueryMatch>,
}

impl Default for QuickSearchState {
    fn default() -> Self {
        Self {
            mode: QuickSearchMode::Off,
            input: String::new(),
            matches: BTreeSet::new(),
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
        // Select first line by default;
        // table.next();

        App {
            title,
            table: LogTable::new(&model),
            state: TableState::default(),
            fps: fps_counter::FPSCounter::new(),
            should_quit: false,
            input_event_message: String::new(),
            quick_search: QuickSearchState::default(),
            vertical_offset: 0,
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

    fn selected(&self) -> usize {
        self.vertical_offset + self.state.selected().unwrap_or(0)
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

        f.render_stateful_widget(t, layout.table, &mut self.state);

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
            "Row {}/{} FPS: {} table built in {}ms, table rendered in {}ms, {} {}",
            self.selected() + 1,
            self.table.len(),
            self.fps.tick(),
            table_built.as_millis(),
            (table_rendered - table_built).as_millis(),
            self.input_event_message,
            match self.quick_search.mode {
                QuickSearchMode::Off => {
                    "".to_string()
                }
                QuickSearchMode::Input | QuickSearchMode::Iteration => {
                    format!("Found {} matches", self.quick_search.matches.len())
                }
            } // if self.quick_search.mode != QuickSearchMode::Off {
              //     format!("Found {} entries", self.quick_search.matches.len())
              // } else {
              //     ""
              // }
        ))
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left);

        f.render_widget(bottom_block, layout.status_bar);
    }

    fn copy_line(&mut self) {
        let selected = self.selected();
        if let Some(entry) = self.table.model.get(selected) {
            self.input_event_message = match ClipboardProvider::new()
                .and_then(|mut ctx: ClipboardContext| ctx.set_contents(format!("{}", entry)))
            {
                Ok(()) => format!("Copied the line {} to clipboard", selected + 1),
                Err(ref e) => format!("Failed to copy line {}", e),
            }
        }
    }

    fn copy_message(&mut self) {
        let selected = self.selected();
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
            KeyCode::Char('/') => {
                self.quick_search.mode = QuickSearchMode::Input;
                self.quick_search.matches.clear();
                if !self.quick_search.input.is_empty() {
                    self.search_and_highlight(&self.quick_search.input.clone());
                }
            }
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
                    self.clear_highlight();
                }
                KeyCode::Enter => {
                    if self.quick_search.input.is_empty() {
                        self.quick_search.mode = QuickSearchMode::Off
                    } else {
                        self.quick_search.mode = QuickSearchMode::Iteration
                    }
                }
                KeyCode::Backspace => {
                    self.quick_search.input.clear();
                    self.clear_highlight();
                }
                KeyCode::Char(c) => {
                    self.quick_search.input.push(c);
                    self.search_and_highlight(&self.quick_search.input.clone());
                }
                _ => {}
            },
            QuickSearchMode::Iteration => match event.code {
                KeyCode::Esc => {
                    self.quick_search.mode = QuickSearchMode::Off;
                    self.clear_highlight();
                }
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

    fn clear_highlight(&self) {
        // todo!()
    }

    fn search_and_highlight(&mut self, query: &str) {
        assert!(!query.is_empty());
        let selected = self.selected();
        {
            let lower_bound = selected.saturating_sub(100);
            let upper_bound = selected.saturating_add(100).max(self.table.len() - 1);

            let matches = &self.quick_search.matches;
            let matches = if matches.is_empty() {
                self.table.display_data[lower_bound..upper_bound]
                    .iter()
                    .enumerate()
                    .filter_map(|(index, entry)| {
                        // todo return both columns
                        if entry.texts[4].contains(query) {
                            Some(QueryMatch::new(index, 4))
                        } else if entry.texts[5].contains(query) {
                            Some(QueryMatch::new(index, 5))
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                matches
                    .iter()
                    .filter(|qm| {
                        let entry = &self.table.display_data[qm.index];
                        entry.texts[qm.column].contains(query)
                    })
                    .cloned()
                    .collect()
            };
            self.quick_search.matches = matches;
        }
    }
}

fn with_ctrl(event: &KeyEvent) -> bool {
    event.modifiers.contains(KeyModifiers::CONTROL)
}
