use crate::search::matches::MatchedLines;

pub struct State {
    pub(crate) mode: Mode,
    pub(crate) input: String,
    pub(crate) results: MatchedLines,
    pub(crate) elapsed: u128,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: Mode::Off,
            input: String::new(),
            results: MatchedLines::default(),
            elapsed: 0,
        }
    }
}

pub enum Mode {
    Off,
    Input,
    Iteration,
}
