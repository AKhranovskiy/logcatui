use crate::loglevel::LogLevel;
use chrono::{DateTime, Utc};
use core::fmt::{Display, Formatter};
use core::result::Result;
use core::result::Result::Ok;
use std::str::FromStr;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub process_id: ProcessID,
    pub thread_id: ThreadID,
    pub log_level: LogLevel,
    pub tag: Tag,
    pub message: Message,
}

type ProcessID = usize;
type ThreadID = usize;
type Tag = String;
type Message = String;

impl Display for LogEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}\t{}\t{}",
            self.timestamp.format("%F %H:%M:%S%.3f"),
            self.process_id,
            self.thread_id,
            self.log_level,
            self.tag,
            self.message
        )
    }
}

impl FromStr for LogEntry {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_ascii_whitespace();
        let date = parts.next();
        let time = parts.next();
        let process_id = parts.next();
        let thread_id = parts.next();
        let level = parts.next();
        let tag = parts.next();
        let message = parts.collect::<Vec<_>>().join(" ");

        let timestamp = date
            .zip(time)
            .and_then(|(date, time)| {
                format!("2021-{}T{}Z", &date, &time)
                    .parse::<DateTime<Utc>>()
                    .ok()
            })
            .ok_or(())?;

        let process_id = process_id
            .and_then(|s| s.parse::<ProcessID>().ok())
            .ok_or(())?;
        let thread_id = thread_id
            .and_then(|s| s.parse::<ThreadID>().ok())
            .ok_or(())?;
        let level = level.and_then(|s| s.parse::<LogLevel>().ok()).ok_or(())?;
        let tag = tag.map(|s| s.trim_end_matches(&[' ', ':'][..])).ok_or(())?;
        let message = message.trim_start_matches(&[' ', ':'][..]);

        Ok(LogEntry {
            timestamp,
            process_id,
            thread_id,
            log_level: level,
            tag: tag.to_string(),
            message: message.to_string(),
        })
    }
}
