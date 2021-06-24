pub mod matches;
pub mod state;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum QuickSearchMode {
    Off,
    Input,
    Iteration,
}
