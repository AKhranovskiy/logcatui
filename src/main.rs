mod events;
mod logentry;
mod loglevel;

use std::io;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Borders, Block, Row, TableState, Table};
use tui::layout::{Layout, Constraint, Rect};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use crate::events::{Event, Events};
use std::error::Error;
use tui::style::{Style, Color, Modifier};
use tui::widgets::Cell;
use unicode_width::UnicodeWidthStr;

pub struct StatefulTable {
    viewport: Rect,
    state: TableState,
    rows: Vec<Vec<String>>,
    constraints: Vec<Constraint>,
    column_offset: usize,
}

impl StatefulTable {
    fn new() -> StatefulTable {
        let rows: Vec<Vec<String>> = (0..10_000).map(|i| {
            vec![
                i.to_string(),
                "2021-08-12 14:14:15.333".to_string(),
                i.to_string(),
                i.to_string(),
                "D".to_string(),
                "SomeApplication".to_string(),
                "Very long string 1. Very long string 2. Very long string 3. Very long string 4. Very long string 5. Very long string 6. Very long string 7. Very long string 8. Very long string 9. Very long string 10.".to_string()
            ]
        }).collect();

        let constraints = rows.iter().fold([0usize; 7].into(), |widths: Vec<usize>, row| {
            row.iter()
                .zip(widths)
                .map(|(s, w): (&String, usize)| w.max(UnicodeWidthStr::width(s.as_str())))
                .collect::<Vec<_>>()
        });

        dbg!(&constraints);

        StatefulTable {
            viewport: Rect::default(),
            state: TableState::default(),
            rows,
            constraints: constraints.iter().map(|w| Constraint::Length(*w as u16)).collect(),
            column_offset: 0,
        }
    }

    pub fn next(&mut self) {
        let next_item = self.state.selected()
            .map(|idx| idx.saturating_add(1).min(self.rows.len() - 1))
            .or(Some(0));
        self.state.select(next_item);
    }

    fn page_size(&self) -> usize {
        self.viewport.height as usize
    }

    pub fn next_page(&mut self) {
        let next_item = self.state.selected()
            .map(|idx| {
                idx.saturating_add(self.page_size()).min(self.rows.len() - 1)
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
                .split(f.size());

            table.viewport = Rect::new(0, 0, f.size().width, f.size().height - 4u16);

            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let normal_style = Style::default().bg(Color::Blue);

            let header_cells = ["#", "Timestamp", "PID", "TID", "Level", "Tag", "Message"]
                .iter()
                .skip(table.column_offset)
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));

            let header = Row::new(header_cells)
                .style(normal_style)
                // .bottom_margin(1)
                ;

            let rows = table.rows.iter().map(|item| {
                let cells = item.iter().skip(table.column_offset).map(|c| Cell::from(c.as_str()));
                Row::new(cells)
            });

            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("Table"))
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .column_spacing(1)
                .widths(&table.constraints[table.column_offset..]);
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
