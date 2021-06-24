use std::collections::BTreeMap;
use std::time::Instant;

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
use crate::search::matches::{Match, MatchedColumn, MatchedLine, MatchedPosition, Matches};
use crate::search::quick::{Mode, State};
use crate::{COLUMN_HEADERS, COLUMN_NUMBER};

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
    quick_search: State,
    height: usize,
    vertical_offset: usize,
    row_heights: BTreeMap<usize, usize>,
}

struct AppLayout {
    table: Rect,
    quick_search: Rect,
    status_bar: Rect,
}

impl<'a> App<'a> {
    pub fn init(title: String, model: &'a [LogEntry]) -> Self {
        App {
            title,
            table: LogTable::new(model),
            state: {
                let mut state = TableState::default();
                state.select(Some(0));
                state
            },
            fps: fps_counter::FPSCounter::new(),
            should_quit: false,
            input_event_message: String::new(),
            quick_search: State::default(),
            height: 0,
            vertical_offset: 0,
            row_heights: BTreeMap::new(),
        }
    }

    fn layout<B: Backend>(&self, f: &mut Frame<B>) -> AppLayout {
        let quick_search_height: u16 = match self.quick_search.mode {
            Mode::Off => 0,
            Mode::Input | Mode::Iteration => 1,
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

    fn visible_range(&self) -> (usize, usize) {
        (self.vertical_offset, self.vertical_offset + self.height)
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let layout = self.layout(f);

        self.table.viewport = Rect::new(0, 0, f.size().width, f.size().height - 4_u16);
        self.height = f.size().height.saturating_sub(4).as_();

        let instant = std::time::Instant::now();

        let header_cells = COLUMN_HEADERS
            .iter()
            .skip(self.table.column_offset)
            .map(|h| Cell::from(*h));

        let header = Row::new(header_cells).style(*STYLE_HEADER);

        let available_message_width = self.table.available_message_width();
        let (start, end) = self.visible_range();
        let mut row_heights = self.row_heights.clone();
        let mut full_height = self.height;
        let rows: Vec<Row> = self
            .table
            .display_data
            .iter()
            .enumerate()
            .skip(start)
            .take(end - start)
            .map(|(index, data)| {
                let (row, height) = data.as_row(
                    self.table.column_offset,
                    available_message_width,
                    self.quick_search.results.exact(index),
                );
                if height > 1 {
                    row_heights.insert(index, height);
                    full_height -= height - 1;
                } else if let Some(h) = row_heights.remove(&index) {
                    full_height += h - 1;
                }
                row
            })
            .collect();
        self.row_heights = row_heights;
        self.height = full_height;

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
            Mode::Off => {}
            Mode::Input => {
                let block = Paragraph::new(format!("/ {}", self.quick_search.input))
                    .block(Block::default().borders(Borders::NONE));
                f.render_widget(block, layout.quick_search);
                let w: u16 = self.quick_search.input.width().as_();
                f.set_cursor(
                    // Put cursor past the end of the input text
                    layout.quick_search.x + 1 + w + 1,
                    layout.quick_search.y,
                );
            }
            Mode::Iteration => {
                let block = Paragraph::new(format!("/{}", self.quick_search.input))
                    .style(*STYLE_QUICK_SEARCH)
                    .block(Block::default().borders(Borders::NONE));
                f.render_widget(block, layout.quick_search);
            }
        }

        let bottom_block = Paragraph::new(format!(
            "Row {}/{} FPS: {} table built in {}ms, table rendered in {}ms, {}",
            self.selected() + 1,
            self.table.len(),
            self.fps.tick(),
            table_built.as_millis(),
            (table_rendered - table_built).as_millis(),
            match self.quick_search.mode {
                Mode::Off | Mode::Input => "".to_string(),
                Mode::Iteration => format!(
                    "found {} matches of \"{}\" for {}ms",
                    self.quick_search.results.len(),
                    self.quick_search.input,
                    self.quick_search.elapsed
                ),
            }
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
        self.should_quit = true;
    }

    fn regular_input(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c') => {
                if with_ctrl(event) {
                    self.quit();
                }
            }
            KeyCode::Down => {
                if let Some(selected) = self.state.selected() {
                    if selected + 1 < self.height {
                        self.state.select(Some(selected + 1));
                    } else {
                        let hiding_row_height =
                            self.row_heights.get(&self.vertical_offset).unwrap_or(&1);
                        self.vertical_offset += hiding_row_height;
                    }
                }
            }
            KeyCode::Up => {
                if let Some(selected) = self.state.selected() {
                    if selected == 0 {
                        let appearing_row_height =
                            self.row_heights.get(&self.vertical_offset).unwrap_or(&1);
                        self.vertical_offset =
                            self.vertical_offset.saturating_sub(*appearing_row_height);
                    } else {
                        self.state.select(Some(selected - 1));
                    }
                }
            }
            KeyCode::PageDown => {
                for _ in 0..self.height {
                    self.regular_input(&KeyEvent::from(KeyCode::Down));
                }
            }
            KeyCode::PageUp => {
                for _ in 0..self.height {
                    self.regular_input(&KeyEvent::from(KeyCode::Up));
                }
            }
            KeyCode::Left => self.table.left(),
            KeyCode::Right => self.table.right(),
            KeyCode::Enter => self.table.wrap_message(self.selected()),
            KeyCode::Char('y') => self.copy_line(),
            KeyCode::Char('Y') => self.copy_message(),
            KeyCode::Home => self.table.column_offset = 0,
            KeyCode::End => self.table.column_offset = COLUMN_NUMBER - 1,
            KeyCode::Char('/') => {
                self.quick_search.mode = Mode::Input;
                self.quick_search.results.clear();
            }
            _ => {}
        }
    }
    pub fn input(&mut self, event: &KeyEvent) {
        self.input_event_message.clear();

        match self.quick_search.mode {
            Mode::Off => self.regular_input(event),
            Mode::Input => match event.code {
                KeyCode::Esc => {
                    self.quick_search.mode = Mode::Off;
                    self.quick_search.input.clear();
                }
                KeyCode::Enter => {
                    if self.quick_search.input.is_empty() {
                        self.quick_search.mode = Mode::Off;
                    } else {
                        self.quick_search.mode = Mode::Iteration;
                        self.update_results();
                        self.jump_to_nearest_result();
                    }
                }
                KeyCode::Backspace => {
                    self.quick_search.input.clear();
                }
                KeyCode::Char(c) => {
                    self.quick_search.input.push(c);
                }
                _ => {}
            },
            Mode::Iteration => match event.code {
                KeyCode::Esc => {
                    self.quick_search.mode = Mode::Off;
                }
                KeyCode::Char('n') => {
                    self.jump_to_next_result();
                }
                KeyCode::Char('N') => {
                    self.jump_to_previous_result();
                }
                _ => self.regular_input(event),
            },
        }
    }

    fn update_results(&mut self) {
        let query = &self.quick_search.input;
        assert!(!query.is_empty());

        let instant = Instant::now();

        self.quick_search.results = if self.quick_search.results.is_empty() {
            self.table
                .display_data
                .iter()
                .enumerate()
                .filter_map(|(line, entry)| {
                    let columns: Vec<MatchedColumn> = entry
                        .texts
                        .iter()
                        .enumerate()
                        .filter_map(|(column, text)| {
                            let positions: Vec<MatchedPosition> = text
                                .match_indices(query)
                                .map(|(index, _)| (index, index + query.len()))
                                .collect();

                            if positions.is_empty() {
                                None
                            } else {
                                Some(MatchedColumn::new(column, &positions))
                            }
                        })
                        .collect();

                    if columns.is_empty() {
                        None
                    } else {
                        Some(MatchedLine::new(line, columns.into()))
                    }
                })
                .collect::<Vec<_>>()
                .into()
        } else {
            self.quick_search
                .results
                .iter()
                .filter_map(|_| None)
                .collect::<Vec<_>>()
                .into()
        };

        self.quick_search.elapsed = instant.elapsed().as_millis();

        if false {
            let triples: Vec<(usize, usize, (usize, usize))> = self
                .quick_search
                .results
                .iter()
                .flat_map(|line| {
                    let li = line.index();
                    line.iter().flat_map(move |column| {
                        let ci = column.index();
                        column.iter().map(move |pos| (li, ci, *pos))
                    })
                })
                .collect();

            let number_of_results = triples.len();

            triples.iter().enumerate().for_each(|(index, triple)| {
                eprintln!(
                    "{}/{} line {} column {} pos ({}:{})",
                    index, number_of_results, triple.0, triple.1, triple.2 .0, triple.2 .1
                );
            });
        }
    }

    fn select(&mut self, line: Option<usize>) {
        if let Some(line) = line {
            if line >= self.visible_range().0 && line <= self.visible_range().1 {
                self.state.select(Some(line - self.vertical_offset));
            } else {
                self.vertical_offset = line;
                self.state.select(Some(0));
            }
        }
    }

    fn jump_to_nearest_result(&mut self) {
        self.select(
            self.quick_search
                .results
                .nearest(self.selected())
                .map(|m| m.index()),
        );
    }

    fn jump_to_next_result(&mut self) {
        self.select(
            self.quick_search
                .results
                .next(self.selected())
                .map(|m| m.index()),
        );
    }

    fn jump_to_previous_result(&mut self) {
        self.select(
            self.quick_search
                .results
                .previous(self.selected())
                .map(|m| m.index()),
        );
    }
}

fn with_ctrl(event: &KeyEvent) -> bool {
    event.modifiers.contains(KeyModifiers::CONTROL)
}
