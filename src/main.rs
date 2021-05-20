use std::{env, error::Error, fs, io, process};

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Layout, Rect};
use tui::layout::Direction::Vertical;
use tui::style::{Color, Modifier, Style};
use tui::Terminal;
use tui::text::Text;
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use unicode_segmentation::UnicodeSegmentation;
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
    widths: Vec<u16>,
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

        let widths = texts.iter().map(|s| UnicodeWidthStr::width(s.as_str()) as u16).collect();

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
            .fold(vec![0u16; COLUMN_NUMBER], |max_widths, data| {
                data.widths.iter()
                    .zip(max_widths)
                    .map(|(w, mw)| *w.max(&mw))
                    .collect()
            });

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

    let header_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(Color::White)
        .bg(Color::DarkGray);
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

            let header_cells = COLUMN_HEADERS.iter()
                .skip(table.column_offset)
                .map(|h| Cell::from(*h));

            let header = Row::new(header_cells).style(header_style);

            let instant = std::time::Instant::now();

            let row_without_message_width = table.column_widths.iter()
                .take(COLUMN_NUMBER - 1)
                .skip(table.column_offset)
                .sum::<u16>();
            let column_spacing: u16 = (COLUMN_NUMBER - table.column_offset) as u16;
            let available_message_width = table.viewport.width - row_without_message_width - column_spacing;

            let rows = table.display_data.iter().map(|data| {
                if data.wrapped && data.widths.last().unwrap() > &available_message_width {
                    let message = data.texts.last().unwrap();
                    let indices = wrap_indices(message, available_message_width);
                    assert!(!indices.is_empty());
                    let lines = split_string_at_indices(message, &indices);
                    let mut line_iter = lines.iter();

                    let mut text = Text::from(*line_iter.next().unwrap());
                    while let Some(line) = line_iter.next() {
                        text.extend(Text::from(*line));
                    }

                    Row::new(
                        data.texts.iter()
                            .take(COLUMN_NUMBER - 1)
                            .skip(table.column_offset)
                            .map(|c| Cell::from(c.as_str()))
                            .chain(std::iter::once(Cell::from(text)))
                    ).height(lines.len() as u16)
                } else {
                    Row::new(
                        data.texts.iter()
                            .skip(table.column_offset)
                            .map(|t| Cell::from(t.as_str()))
                    )
                }
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

fn wrap_indices(text: &str, max_width: u16) -> Vec<u16> {
    let mut word_indices = text
        .split_word_bound_indices()
        .map(|(pos, _)| pos as u16);

    let mut lines = vec![];
    let mut prev = None;
    let mut len = max_width;

    while let Some(pos) = word_indices.next() {
        if pos > len {
            if let Some(prev) = prev {
                lines.push(prev)
            }
            prev = Some(pos);
            len += max_width;
        } else {
            prev = Some(pos)
        }
    }

    lines
}

#[test]
fn wrap_long_text() {
    let text = concat!(
    "Explicit concurrent copying GC freed 47311(2322KB) AllocSpace objects, ",
    "17(724KB) LOS objects, 49% free, 12MB/25MB, paused 339us total 141.468ms"
    );

    assert_eq!(wrap_indices(text, 20), vec![20, 37, 51, 80, 98, 115]);
}

#[test]
fn wrap_short_text() {
    let text = "Lorem ipsum";

    assert_eq!(wrap_indices(text, 20), vec![]);
}

#[test]
fn wrap_empty_text() {
    let text = "";

    assert_eq!(wrap_indices(text, 20), vec![]);
}

fn split_string_at_indices<'a>(s: &'a str, indices: &[u16]) -> Vec<&'a str> {
    assert!((*indices.iter().max().unwrap_or(&0) as usize) < s.len());

    let mut off = 0u16;
    let mut ms = s;
    let mut parts: Vec<&str> = indices.iter().map(|&index| {
        let (head, tail) = ms.split_at((index - off) as usize);
        off = index;
        ms = tail;
        head
    }).collect();
    parts.push(ms);
    parts
}

#[test]
fn test_split_string_at_indices() {
    let s = concat!(
    "Explicit concurrent copying GC freed 47311(2322KB) AllocSpace objects, ",
    "17(724KB) LOS objects, 49% free, 12MB/25MB, paused 339us total 141.468ms"
    );
    let indices = wrap_indices(s, 20);

    let splits = split_string_at_indices(s, &indices);

    assert_eq!(splits, vec!["Explicit concurrent ",
                            "copying GC freed ",
                            "47311(2322KB) ",
                            "AllocSpace objects, 17(724KB)",
                            " LOS objects, 49% ",
                            "free, 12MB/25MB, ",
                            "paused 339us total 141.468ms"]);
}

#[test]
fn test_split_string_at_no_indices() {
    let s = "Explicit concurrent copying";
    let splits = split_string_at_indices(s, &[]);

    assert_eq!(splits, vec!["Explicit concurrent copying"]);
}

#[test]
fn test_split_suspicious() {
    let s = concat!(
    "Invalidating LocalCallingIdentity cache for package ",
    "com.tomtom.ivi.functionaltest.frontend.alexa.test. ",
    "Reason: package android.intent.action.PACKAGE_REMOVED"
    );
    // It splits in 3 parts of strange lenghts, while it shall be more.
    let indices = wrap_indices(s, 50);

    assert_eq!(indices, vec![50, 100, 150]);
}
/*
 */
