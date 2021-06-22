use std::collections::BTreeSet;

pub type MatchedPosition = (usize, usize);

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct MatchedColumn {
    pub(crate) index: usize,
    pub(crate) positions: BTreeSet<MatchedPosition>,
}

impl MatchedColumn {
    pub(crate) fn new(index: usize, positions: &[MatchedPosition]) -> Self {
        MatchedColumn {
            index,
            positions: positions.iter().copied().collect(),
        }
    }
}

#[derive(Eq, PartialEq, PartialOrd, Ord)]
pub struct MatchedLine {
    pub(crate) index: usize,
    pub(crate) columns: BTreeSet<MatchedColumn>,
}

impl MatchedLine {
    pub(crate) fn new(index: usize, columns: &[MatchedColumn]) -> Self {
        MatchedLine {
            index,
            columns: columns.iter().cloned().collect(),
        }
    }
}

pub struct QuickSearchState {
    pub(crate) mode: QuickSearchMode,
    pub(crate) input: String,
    pub(crate) results: BTreeSet<MatchedLine>,
    pub(crate) elapsed: u128,
}

impl Default for QuickSearchState {
    fn default() -> Self {
        Self {
            mode: QuickSearchMode::Off,
            input: String::new(),
            results: BTreeSet::new(),
            elapsed: 0,
        }
    }
}

pub enum QuickSearchMode {
    Off,
    Input,
    Iteration,
}
