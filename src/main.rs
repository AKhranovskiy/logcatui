#![feature(result_flattening)]

use std::time::Duration;
use std::{env, error::Error, fs, io, process};

use crossterm::event::{poll, read, Event};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::CrosstermBackend;
use tui::Terminal;

use crate::logentry::LogEntry;

mod app;
mod display_data;
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

    execute!(io::stdout(), EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| app.draw(f))?;

        while (poll(Duration::from_millis(0)))? {
            match read()? {
                Event::Key(event) => app.input(&event),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
