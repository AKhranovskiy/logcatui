use std::collections::btree_set::Iter;
use std::collections::BTreeSet;

use crate::search::matches::MatchedLine;

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
// fn closest(target: usize, lower: Option<usize>, upper: Option<usize>) -> Option<usize> {
//     let ld = distance(target, lower.unwrap_or(usize::MAX));
//     let ud = distance(target, upper.unwrap_or(usize::MAX));
//
//     if ld < ud {
//         lower
//     } else {
//         upper
//     }
// }
//
// #[test]
// fn test_closest() {
//     assert_eq!(None, closest(0, None, None));
//     assert_eq!(Some(1), closest(0, Some(1), None));
//     assert_eq!(Some(1), closest(0, None, Some(1)));
//     assert_eq!(Some(1), closest(0, Some(1), Some(2)));
// }

// #[test]
// fn is_line_matched() {
//     use std::collections::BTreeSet;
//
//     let sut: BTreeSet<MatchedLine> = [
//         MatchedLine::new(1, &[]),
//         MatchedLine::new(3, &[]),
//         MatchedLine::new(5, &[]),
//     ]
//     .as_ref()
//     .iter()
//     .cloned()
//     .collect();
//
//     let sentinel = |index: usize| MatchedLine::new(index, &[]);
//     let lookup = |index: usize| {
//         sut.range((Included(sentinel(index)), Excluded(sentinel(index + 1))))
//             .next()
//             .map(|line| line.index)
//     };
//
//     assert_eq!(None, lookup(0));
//     assert_eq!(Some(1), lookup(1));
//     assert_eq!(None, lookup(2));
//     assert_eq!(Some(3), lookup(3));
//     assert_eq!(None, lookup(6));
// }
