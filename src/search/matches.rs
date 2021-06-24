use std::collections::btree_set::Iter;
use std::collections::BTreeSet;

pub trait Match: Ord + PartialOrd + Eq + PartialEq {
    fn index(&self) -> usize;
    fn sentinel(index: usize) -> Self;
}

trait Matches<T: Match> {
    fn nearest(&self, index: usize) -> Option<&T>;
    fn previous(&self, index: usize) -> Option<&T>;
    fn next(&self, index: usize) -> Option<&T>;
    fn exact(&self, index: usize) -> Option<&T>;
}

pub struct MatchedLines {
    lines: BTreeSet<MatchedLine>,
}

impl From<&[MatchedLine]> for MatchedLines {
    fn from(lines: &[MatchedLine]) -> Self {
        Self {
            lines: lines.iter().cloned().collect(),
        }
    }
}

impl Matches<MatchedLine> for MatchedLines {
    fn nearest(&self, index: usize) -> Option<&MatchedLine> {
        implementation::nearest_match(&self.lines, index)
    }

    fn previous(&self, index: usize) -> Option<&MatchedLine> {
        implementation::previous_match(&self.lines, index)
    }

    fn next(&self, index: usize) -> Option<&MatchedLine> {
        implementation::next_match(&self.lines, index)
    }

    fn exact(&self, index: usize) -> Option<&MatchedLine> {
        implementation::exact_match(&self.lines, index)
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct MatchedLine {
    pub(crate) index: usize,
    pub(crate) columns: MatchedColumns,
}

impl MatchedLine {
    pub(crate) fn new(index: usize, columns: MatchedColumns) -> Self {
        MatchedLine { index, columns }
    }
}

impl Match for MatchedLine {
    fn index(&self) -> usize {
        self.index
    }

    fn sentinel(index: usize) -> Self {
        Self {
            index,
            columns: Default::default(),
        }
    }
}

#[derive(Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MatchedColumns {
    columns: BTreeSet<MatchedColumn>,
}

impl From<Vec<MatchedColumn>> for MatchedColumns {
    fn from(columns: Vec<MatchedColumn>) -> Self {
        Self {
            columns: columns.iter().cloned().collect(),
        }
    }
}

impl MatchedColumns {
    #[inline]
    pub fn iter(&self) -> Iter<MatchedColumn> {
        self.columns.iter()
    }
}

impl Matches<MatchedColumn> for MatchedColumns {
    fn nearest(&self, index: usize) -> Option<&MatchedColumn> {
        todo!()
    }

    fn previous(&self, index: usize) -> Option<&MatchedColumn> {
        todo!()
    }

    fn next(&self, index: usize) -> Option<&MatchedColumn> {
        todo!()
    }

    fn exact(&self, index: usize) -> Option<&MatchedColumn> {
        todo!()
    }
}

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

impl Match for MatchedColumn {
    fn index(&self) -> usize {
        self.index
    }

    fn sentinel(index: usize) -> Self {
        Self {
            index,
            positions: BTreeSet::default(),
        }
    }
}

pub type MatchedPosition = (usize, usize);

mod implementation {
    use std::cmp::Ordering;
    use std::collections::BTreeSet;
    use std::ops::Bound::{Excluded, Included, Unbounded};

    use crate::search::matches::Match;

    fn distance(a: usize, b: usize) -> usize {
        match a.cmp(&b) {
            Ordering::Less => b - a,
            Ordering::Equal => 0,
            Ordering::Greater => a - b,
        }
    }

    pub(crate) fn nearest_match<T: Match>(matches: &BTreeSet<T>, index: usize) -> Option<&T> {
        let sentinel = T::sentinel(index);
        let lower = matches.range((Unbounded, Included(&sentinel))).next_back();
        let upper = matches.range((Included(&sentinel), Unbounded)).next();

        if distance(sentinel.index(), lower.map_or(usize::MAX, |m| m.index()))
            <= distance(sentinel.index(), upper.map_or(usize::MAX, |m| m.index()))
        {
            lower
        } else {
            upper
        }
    }

    pub(crate) fn previous_match<T: Match>(matches: &BTreeSet<T>, index: usize) -> Option<&T> {
        matches
            .range((Unbounded, Excluded(&T::sentinel(index))))
            .next_back()
    }

    pub(crate) fn next_match<T: Match>(matches: &BTreeSet<T>, index: usize) -> Option<&T> {
        matches
            .range((Included(&T::sentinel(index + 1)), Unbounded))
            .next()
    }

    pub(crate) fn exact_match<T: Match>(matches: &BTreeSet<T>, index: usize) -> Option<&T> {
        matches
            .range((
                Included(&T::sentinel(index)),
                Excluded(&T::sentinel(index + 1)),
            ))
            .next()
    }

    #[cfg(test)]
    mod tests {
        use std::collections::BTreeSet;

        use crate::search::matches::implementation::{
            exact_match, nearest_match, next_match, previous_match,
        };
        use crate::search::matches::Match;

        #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
        struct TestMatch {
            index: usize,
        }

        impl Match for TestMatch {
            fn index(&self) -> usize {
                self.index
            }

            fn sentinel(index: usize) -> Self {
                Self { index }
            }
        }

        #[test]
        fn test_nearest_match() {
            let first = TestMatch::sentinel(1);
            let second = TestMatch::sentinel(4);
            let matches: BTreeSet<TestMatch> = [first.clone(), second.clone()]
                .as_ref()
                .iter()
                .cloned()
                .collect();

            assert_eq!(None, nearest_match::<TestMatch>(&BTreeSet::default(), 0));
            assert_eq!(Some(&first), nearest_match(&matches, 0));
            assert_eq!(Some(&second), nearest_match(&matches, 3));
            assert_eq!(Some(&second), nearest_match(&matches, 100));
        }

        #[test]
        fn test_previous_match() {
            let first = TestMatch::sentinel(1);
            let second = TestMatch::sentinel(4);
            let third = TestMatch::sentinel(6);
            let matches: BTreeSet<TestMatch> = [first.clone(), second.clone(), third.clone()]
                .as_ref()
                .iter()
                .cloned()
                .collect();

            assert_eq!(None, previous_match::<TestMatch>(&BTreeSet::default(), 0));
            assert_eq!(None, previous_match::<TestMatch>(&BTreeSet::default(), 100));
            assert_eq!(None, previous_match(&matches, 0));
            assert_eq!(None, previous_match(&matches, 1));
            assert_eq!(Some(&first), nearest_match(&matches, 2));
            assert_eq!(Some(&first), previous_match(&matches, 4));
            assert_eq!(Some(&second), previous_match(&matches, 5));
            assert_eq!(Some(&third), previous_match(&matches, 100));
        }

        #[test]
        fn test_next_match() {
            let first = TestMatch::sentinel(1);
            let second = TestMatch::sentinel(4);
            let third = TestMatch::sentinel(6);
            let matches: BTreeSet<TestMatch> = [first.clone(), second.clone(), third.clone()]
                .as_ref()
                .iter()
                .cloned()
                .collect();

            assert_eq!(None, next_match::<TestMatch>(&BTreeSet::default(), 0));
            assert_eq!(None, next_match::<TestMatch>(&BTreeSet::default(), 100));
            assert_eq!(Some(&first), next_match(&matches, 0));
            assert_eq!(Some(&second), next_match(&matches, 1));
            assert_eq!(Some(&third), next_match(&matches, 4));
            assert_eq!(None, next_match(&matches, 100));
        }

        #[test]
        fn test_exact_match() {
            let first = TestMatch::sentinel(1);
            let second = TestMatch::sentinel(4);
            let matches: BTreeSet<TestMatch> = [first.clone(), second.clone()]
                .as_ref()
                .iter()
                .cloned()
                .collect();

            assert_eq!(None, exact_match::<TestMatch>(&BTreeSet::default(), 0));
            assert_eq!(None, exact_match(&matches, 0));
            assert_eq!(Some(&first), exact_match(&matches, 1));
            assert_eq!(Some(&second), exact_match(&matches, 4));
        }
    }
}
