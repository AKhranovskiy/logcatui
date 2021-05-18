mod events;

use std::io;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Borders, Block};
use tui::layout::{Layout, Direction, Constraint};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use crate::events::{Event, Events};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let events = Events::new();

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10)
                    ].as_ref()
                )
                .split(f.size());
            let block = Block::default()
                .title("Block")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[0]);
            let block = Block::default()
                .title("Block 2")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[1]);
        })?;

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => {
                    break;
                }
                _ => {}
            }
        };
    }

    Ok(())
}
