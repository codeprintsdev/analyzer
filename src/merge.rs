use std::convert::TryFrom;

use crate::{parser::ParseState, types::Timeline};
use anyhow::Result;

/// Merger merges multiple timelines into one
#[derive(Debug)]
pub struct Merger {
    state: ParseState,
}

impl Merger {
    /// Create a new merger with a clean state (no timeline merged)
    pub fn new() -> Self {
        let state = ParseState::default();
        Merger { state }
    }

    fn merge_single(&mut self, single: ParseState) -> Result<()> {
        self.state = single;
        Ok(())
    }

    /// Merge multiple timelines together into one
    /// This is helpful when analyzing multiple repositories and trying to combine
    /// the individual results.
    pub fn merge(&mut self, timelines: &[Timeline]) -> Result<Timeline> {
        for timeline in timelines {
            let single = ParseState::try_from(timeline.clone())?;
            self.merge_single(single)?;
        }
        Ok(Timeline::try_from(&self.state)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{Contribution, Range, Year};

    #[test]
    fn test_merge_none() {
        let mut merger = Merger::new();
        assert_eq!(merger.merge(&[]).unwrap(), Timeline::default());
    }

    #[test]
    fn test_merge_one() {
        let mut timeline = Timeline::default();

        let year = "2020".into();
        let total = 1234;
        let range = Range {
            start: "2020-02-01".into(),
            end: "2020-03-02".into(),
        };

        let year1 = Year { year, total, range };
        let years = vec![year1];
        timeline.years = years;

        let contributions = vec![
            Contribution {
                date: "2020-01-01".into(),
                count: 1000,
                color: "".into(),
                intensity: 4,
            },
            Contribution {
                date: "2020-01-02".into(),
                count: 234,
                color: "".into(),
                intensity: 4,
            },
        ];

        timeline.contributions = contributions;

        let mut merger = Merger::new();
        assert_eq!(merger.merge(&[timeline.clone()]).unwrap(), timeline);
    }
}
