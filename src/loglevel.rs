use core::fmt::{Display, Formatter};
use core::option::Option::{None, Some};
use core::result::Result;
use std::str::FromStr;

#[derive(Debug)]
pub enum LogLevel {
    Verbose,
    Debug,
    Warning,
    Info,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            LogLevel::Verbose => 'V',
            LogLevel::Debug => 'D',
            LogLevel::Warning => 'W',
            LogLevel::Info => 'I',
            LogLevel::Error => 'E'
        })
    }
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars().next().map(|c| {
            match c.to_uppercase().next() {
                Some('V') => Some(LogLevel::Verbose),
                Some('D') => Some(LogLevel::Debug),
                Some('W') => Some(LogLevel::Warning),
                Some('I') => Some(LogLevel::Info),
                Some('E') => Some(LogLevel::Error),
                _ => None
            }
        }).flatten().ok_or(())
    }
}
