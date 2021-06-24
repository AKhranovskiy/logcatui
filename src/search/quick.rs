use crate::search::matches::{MatchedColumn, MatchedLine, MatchedLines, MatchedPosition};

pub struct State {
    mode: Mode,
    input: String,
    results: MatchedLines,
    pub(crate) elapsed: u128,
}

impl State {
    pub fn update<'a>(
        &mut self,
        texts: impl Iterator<Item = (impl Iterator<Item = &'a str> + 'a)>,
    ) -> usize {
        if query.is_empty() {
            self.results.clear();
            0
        } else {
            self.results = texts
                .enumerate()
                .filter_map(|(line, cells)| {
                    let columns: Vec<MatchedColumn> = cells
                        .enumerate()
                        .filter_map(|(column, text)| {
                            let positions: Vec<MatchedPosition> = text
                                .match_indices(self.input())
                                .map(|(index, _)| (index, index + self.input().len()))
                                .collect();

                            if positions.is_empty() {
                                None
                            } else {
                                Some(MatchedColumn::new(column, &positions))
                            }
                        })
                        .collect();
                    if columns.is_empty() {
                        None
                    } else {
                        Some(MatchedLine::new(line, columns.into()))
                    }
                })
                .collect::<Vec<_>>()
                .into();

            self.results.len()
        }
    }

    pub fn results(&self) -> &MatchedLines {
        &self.results
    }

    pub fn input(&self) -> &String {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut String {
        &mut self.input
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
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
