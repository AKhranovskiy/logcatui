mod events;

use std::io;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Borders, Block, Row, TableState, Table};
use tui::layout::{Layout, Direction, Constraint, Rect};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use crate::events::{Event, Events};
use std::error::Error;
use tui::style::{Style, Color, Modifier};
use tui::widgets::Cell;

pub struct StatefulTable {
    viewport: Rect,
    state: TableState,
    items: Vec<Vec<String>>,
    column_offset: usize,
}

impl StatefulTable {
    fn new() -> StatefulTable {
        StatefulTable {
            viewport: Rect::default(),
            state: TableState::default(),
            items: (0..10_000).map(|i| {
                vec![
                    format!("Row {}-1", i + 1).to_string(),
                    format!("Row {}-2", i + 1).to_string(),
                    format!("Row {}-3", i + 1).to_string(),
                    format!("Row {}-4", i + 1).to_string(),
                    format!("Row {}-5", i + 1).to_string(),
                    format!("Row {}-6", i + 1).to_string(),
                ]
            }).collect(),
            column_offset: 0,
        }
    }

    pub fn next(&mut self) {
        let next_item = self.state.selected()
            .map(|idx| idx.saturating_add(1).min(self.items.len() - 1))
            .or(Some(0));
        self.state.select(next_item);
    }

    fn page_size(&self) -> usize {
        self.viewport.height as usize
    }

    pub fn next_page(&mut self) {
        let next_item = self.state.selected()
            .map(|idx| {
                idx.saturating_add(self.page_size()).min(self.items.len() - 1)
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
        self.column_offset = self.column_offset.saturating_add(1).min(5)
    }
    pub fn left(&mut self) {
        self.column_offset = self.column_offset.saturating_sub(1)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let events = Events::new();
    let mut table = StatefulTable::new();

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                // .margin(1)
                .split(f.size());

            table.viewport = Rect::new(0, 0, f.size().width, f.size().height - 4u16);

            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let normal_style = Style::default().bg(Color::Blue);

            let header_cells = ["Header1", "Header2", "Header3", "Header4", "Header5", "Header6"]
                .iter()
                .skip(table.column_offset)
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));

            let header = Row::new(header_cells)
                .style(normal_style)
                .bottom_margin(1);

            let rows = table.items.iter().map(|item| {
                let cells = item.iter().skip(table.column_offset).map(|c| Cell::from(c.as_str()));
                Row::new(cells)
            });

            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("Table"))
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .widths(&[
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(70),
                ][table.column_offset..]);
            f.render_stateful_widget(t, rects[0], &mut table.state);
        })?;

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => {
                    break;
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
                _ => {}
            }
        };
    }

    Ok(())
}
