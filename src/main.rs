use std::{env, error::Error, fs, io, process};

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Layout, Rect};
use tui::layout::Direction::Vertical;
use tui::style::{Color, Modifier, Style};
use tui::Terminal;
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
// use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

use crate::events::{Event, Events};
use crate::logentry::LogEntry;

mod events;
mod logentry;
mod loglevel;

#[allow(dead_code)]
pub struct DisplayData<'a> {
    log_entry: &'a LogEntry,
    texts: Vec<String>,
    widths: Vec<usize>,
    wrapped: bool,
}

const COLUMN_NUMBER: usize = 6;
const COLUMN_HEADERS: [&str; COLUMN_NUMBER] = [
    "Timestamp", "PID", "TID", "Level", "Tag", "Message"
];

impl<'a> DisplayData<'a> {
    fn new(entry: &'a LogEntry) -> Self {
        let texts = vec![
            entry.timestamp.format("%F %H:%M:%S%.3f").to_string(),
            entry.process_id.to_string(),
            entry.thread_id.to_string(),
            entry.log_level.to_string(),
            entry.tag.to_string(),
            entry.message.to_string(),
        ];
        assert_eq!(texts.len(), COLUMN_NUMBER);

        let widths = texts.iter().map(|s| UnicodeWidthStr::width(s.as_str())).collect();

        DisplayData {
            log_entry: entry,
            texts,
            widths,
            wrapped: false,
        }
    }
}

pub struct StatefulTable<'a> {
    state: TableState,
    model: &'a Vec<LogEntry>,
    display_data: Vec<DisplayData<'a>>,
    column_widths: Vec<u16>,
    viewport: Rect,
    column_offset: usize,
}

impl<'a> StatefulTable<'a> {
    fn new(model: &'a Vec<LogEntry>) -> StatefulTable {
        let display_data: Vec<DisplayData> = model.iter()
            .map(|entry| DisplayData::new(entry))
            .collect();

        let mut column_widths = display_data.iter()
            .fold(vec![0usize; COLUMN_NUMBER], |max_widths, data| {
                data.widths.iter().zip(max_widths).map(|(w, mw)| *w.max(&mw)).collect()
            })
            .iter()
            .map(|w| *w as u16)
            .collect::<Vec<_>>();

        // Override width of TAG column because the maximum lenght is almost always too much.
        column_widths[4] = 18;

        StatefulTable {
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

    pub fn next(&mut self) {
        let next_item = self.state.selected()
            .map(|idx| idx.saturating_add(1).min(self.len() - 1))
            .or(Some(0));
        self.state.select(next_item);
    }

    fn page_size(&self) -> usize {
        self.viewport.height as usize
    }

    pub fn next_page(&mut self) {
        let next_item = self.state.selected()
            .map(|idx| {
                idx.saturating_add(self.page_size()).min(self.len() - 1)
            })
            .or(Some(0));
        self.state.select(next_item);
    }

    pub fn previous(&mut self) {
        let prev_item = self.state.selected()
            .map(|idx| idx.saturating_sub(1))
            .or(Some(0));
        self.state.select(prev_item);
    }
    pub fn previous_page(&mut self) {
        let prev_item = self.state.selected()
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
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("No input file specified");
        process::exit(1)
    });

    let input =
        fs::read_to_string(&input_file).expect(&format!("Failed to read file {}", &input_file));

    let start = std::time::Instant::now();
    let model = input
        .lines()
        .filter_map(|line| line.parse().ok())
        .collect::<Vec<LogEntry>>();

    println!(
        "Parsed {} entries, elapsed {}ms",
        model.len(),
        start.elapsed().as_millis()
    );

    let events = Events::new();
    let mut table = StatefulTable::new(&model);

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let normal_style = Style::default().bg(Color::Blue);
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let mut fps_counter = fps_counter::FPSCounter::new();

    // Select the first line by default.
    table.next();

    'main_loop: loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(1)
                ].as_ref())
                .split(f.size());

            table.viewport = Rect::new(0, 0, f.size().width, f.size().height - 4u16);

            // let row_without_message_width = table.column_constraints.iter()
            //     .take(table.column_constraints.len() - 1)
            //     .skip(table.column_offset)
            //     .map(|v| if let Constraint::Length(l) = v { *l } else { 0u16 })
            //     .sum::<u16>();

            let header_cells = COLUMN_HEADERS
                .iter()
                .skip(table.column_offset)
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));

            let header = Row::new(header_cells).style(normal_style);

            let instant = std::time::Instant::now();

            let rows = table.display_data.iter().map(|data| {
                Row::new(
                    data.texts.iter()
                        .skip(table.column_offset)
                        .map(|t| Cell::from(t.as_str()))
                )
                // if table.wrapped.contains(&idx) {
                //     let message = row.last().unwrap();
                //     let message_length = UnicodeWidthStr::width(message.as_str()) as u16;
                //     let selector_width = 3u16;
                //     let column_spacing: u16 = (table.constraints.len() - table.column_offset) as u16;
                //     let available_message_width = table.viewport.width - row_without_message_width - selector_width - column_spacing;
                //     if message_length > available_message_width {
                //         let graphemes =
                //             UnicodeSegmentation::graphemes(message.as_str(), true)
                //                 .collect::<Vec<&str>>();
                //
                //         let chunks = graphemes.chunks(available_message_width as usize - 1);
                //         let height = chunks.len();
                //
                //         let message = chunks.map(|s| s.join("")).fold(
                //             String::with_capacity(message_length as usize + height),
                //             |mut r: String, c| {
                //                 r.push_str(c.as_str());
                //                 r.push('\n');
                //                 r
                //             },
                //         );
                //         Row::new(row.iter()
                //             .skip(table.column_offset)
                //             .take(table.constraints.len() - 1 - table.column_offset)
                //             .map(|c| Cell::from(c.as_str()))
                //             .chain(std::iter::once(Cell::from(message.to_string()))))
                //             .height(height as u16)
                //     } else {
                //         Row::new(row.iter()
                //             .skip(table.column_offset)
                //             .map(|c| Cell::from(c.as_str())))
                //     }
                // } else {
                //     Row::new(row.iter()
                //         .skip(table.column_offset)
                //         .map(|c| Cell::from(c.as_str())))
                // }
            });

            let constraints = table.column_widths[table.column_offset..]
                .iter()
                .map(|&w| Constraint::Length(w))
                .collect::<Vec<_>>();

            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title(input_file.as_str()))
                .highlight_style(selected_style)
                .column_spacing(1)
                .widths(&constraints);

            let table_built = instant.elapsed();

            f.render_stateful_widget(t, chunks[0], &mut table.state);
            let table_rendered = instant.elapsed();

            let bottom_block = Paragraph::new(
                format!(
                    "Row {}/{} FPS: {} table built in {}ms, table rendered in {}ms",
                    table.state.selected().map(|v| v + 1).unwrap_or(0),
                    table.len(),
                    fps_counter.tick(),
                    table_built.as_millis(),
                    (table_rendered - table_built).as_millis()
                )
            )
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Left);
            f.render_widget(bottom_block, chunks[1]);
        })?;

        for event in events.next_batch() {
            if let Event::Input(key) = event {
                match key {
                    Key::Char('q') | Key::Ctrl('c') => {
                        break 'main_loop;
                    }
                    Key::Down => {
                        table.next();
                    }
                    Key::Up => {
                        table.previous();
                    }
                    Key::PageDown => {
                        table.next_page();
                    }
                    Key::PageUp => {
                        table.previous_page();
                    }
                    Key::Left => { table.left(); }
                    Key::Right => { table.right(); }
                    Key::Char('\n') => { table.wrap_message(); }
                    _ => {
                        // dbg!(key);
                    }
                }
            }
        }
    }

    Ok(())
}
