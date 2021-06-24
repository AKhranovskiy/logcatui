use std::collections::BTreeSet;

use crate::search::matches::MatchedLine;

pub struct State {
    pub(crate) mode: Mode,
    pub(crate) input: String,
    pub(crate) results: BTreeSet<MatchedLine>,
    pub(crate) elapsed: u128,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: Mode::Off,
            input: String::new(),
            results: BTreeSet::new(),
            elapsed: 0,
        }
    }
}

pub enum Mode {
    Off,
    Input,
    Iteration,
}
