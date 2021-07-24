use tui::style::{Color, Modifier, Style};

lazy_static! {
    pub static ref STYLE_HEADER: Style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(Color::White)
        .bg(Color::DarkGray);
    pub static ref STYLE_SELECTED_ROW: Style = Style::default().add_modifier(Modifier::REVERSED);
    pub static ref STYLE_QUICK_SEARCH: Style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    pub static ref STYLE_SEARCH_HIGHLIGHT: Style =
        Style::default().fg(Color::Yellow).bg(Color::Blue);
    pub static ref STYLE_LOGLEVEL_ERROR: Style = Style::default().bg(Color::Red);
    pub static ref STYLE_LOGLEVEL_INFO: Style = Style::default().bg(Color::LightRed);
    pub static ref STYLE_LOGLEVEL_WARNING: Style = Style::default().bg(Color::Magenta);
    pub static ref STYLE_LOGLEVEL_DEBUG: Style = Style::default().bg(Color::Cyan);
    pub static ref STYLE_LOGLEVEL_VERBOSE: Style = Style::default().bg(Color::Gray);
}
