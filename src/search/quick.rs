use crate::search::matches::MatchedLines;

pub struct State {
    mode: Mode,
    pub(crate) input: String,
    pub(crate) results: MatchedLines,
    pub(crate) elapsed: u128,
}

impl State {
    pub(crate) fn mode(&self) -> Mode {
        self.mode
    }

    pub(crate) fn set_mode(&mut self, mode: Mode) {
        if self.mode == Mode::Off && mode == Mode::Input {
            self.results.clear();
        }
        if self.mode == Mode::Input && mode == Mode::Off {
            self.input.clear();
        }
        if self.mode == Mode::Input && mode == Mode::Iteration {
            // update results
        }
        // if self.quick_search.input.is_empty() {
        //     self.quick_search.mode = Mode::Off;
        // } else {
        //     self.quick_search.mode = Mode::Iteration;
        //     self.update_results();
        //     self.jump_to_nearest_result();
        // }
        self.mode = mode;
    }
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

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    Off,
    Input,
    Iteration,
}
