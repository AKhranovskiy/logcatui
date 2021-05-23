use std::{env, error::Error, fs, io, process};

use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::backend::TermionBackend;
use tui::Terminal;

use crate::events::{Event, Events};
use crate::logentry::LogEntry;

mod app;
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

    let mut app = app::App::init(input_file, &model);

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    'main_loop: loop {
        terminal.draw(|f| app.draw(f))?;

        for event in events.next_batch() {
            if let Event::Input(key) = event {
                app.input(&key);
                if app.should_quit {
                    break 'main_loop;
                }
            }
        }
    }

    Ok(())
}
