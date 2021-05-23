use std::{env, error::Error, fs, io, process};

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::backend::TermionBackend;
use tui::layout::Direction::Vertical;
use tui::layout::{Alignment, Constraint, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use tui::Terminal;

use log_table::LogTable;

use crate::events::{Event, Events};
use crate::logentry::LogEntry;

mod display_data;
mod events;
mod log_table;
mod logentry;
mod loglevel;
mod text_utils;

const COLUMN_NUMBER: usize = 6;
const COLUMN_HEADERS: [&str; COLUMN_NUMBER] =
    ["Timestamp", "PID", "TID", "Level", "Tag", "Message"];

fn load_logfile(input_file: &str) -> Vec<LogEntry> {
    let input = fs::read_to_string(&input_file)
        .unwrap_or_else(|_| panic!("Failed to read file {}", &input_file));

    let model = input
        .lines()
        .filter_map(|line| line.parse().ok())
        .collect::<Vec<LogEntry>>();

    model
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("No input file specified");
        process::exit(1)
    });

    let start = std::time::Instant::now();

    let model = load_logfile(&input_file);

    println!(
        "Parsed {} entries, elapsed {}ms",
        model.len(),
        start.elapsed().as_millis()
    );

    let events = Events::new();
    let mut table = LogTable::new(&model);

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
                .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
                .split(f.size());

            table.viewport = Rect::new(0, 0, f.size().width, f.size().height - 4u16);

            let header_cells = COLUMN_HEADERS
                .iter()
                .skip(table.column_offset)
                .map(|h| Cell::from(*h));

            let header = Row::new(header_cells).style(header_style);

            let instant = std::time::Instant::now();

            let available_message_width = table.available_message_width();

            let rows = table
                .display_data
                .iter()
                .map(|data| data.as_row(table.column_offset, available_message_width));

            let constraints = table.column_widths[table.column_offset..]
                .iter()
                .map(|&w| Constraint::Length(w))
                .collect::<Vec<_>>();

            let t = Table::new(rows)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(input_file.as_str()),
                )
                .highlight_style(selected_style)
                .column_spacing(1)
                .widths(&constraints);

            let table_built = instant.elapsed();

            f.render_stateful_widget(t, chunks[0], &mut table.state);

            let table_rendered = instant.elapsed();

            let bottom_block = Paragraph::new(format!(
                "Row {}/{} FPS: {} table built in {}ms, table rendered in {}ms",
                table.state.selected().map(|v| v + 1).unwrap_or(0),
                table.len(),
                fps_counter.tick(),
                table_built.as_millis(),
                (table_rendered - table_built).as_millis()
            ))
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
                    Key::Left => {
                        table.left();
                    }
                    Key::Right => {
                        table.right();
                    }
                    Key::Char('\n') => {
                        table.wrap_message();
                    }
                    _ => {
                        // dbg!(key);
                    }
                }
            }
        }
    }

    Ok(())
}
