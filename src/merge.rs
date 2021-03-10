use std::convert::TryFrom;

use crate::{git, parser::ParseState, types::Timeline};
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

    /// Merge a single timeline
    pub fn merge_timeline(&mut self, timeline: &Timeline) -> Result<()> {
        // TODO: make sure that all days are filled with 0 between start and end
        for contribution in &timeline.contributions {
            let date = contribution.date.clone();
            let date = git::parse_date(&date)?;
            let count = contribution.count;

            if let Some(date) = date {
                self.state.update_years(date, count);
                self.state.update_days(date, count);
            }
        }
        Ok(())
    }

    /// Merge multiple timelines together into one
    /// This is helpful when analyzing multiple repositories and trying to combine
    /// the individual results.
    pub fn merge(&mut self, timelines: &[Timeline]) -> Result<Timeline> {
        for timeline in timelines {
            self.merge_timeline(timeline)?
        }
        Ok(Timeline::try_from(&self.state)?)
    }

    /// Return the merged timeline of all inputs
    pub fn timeline(&self) -> Result<Timeline> {
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
            start: "2020-01-01".into(),
            end: "2020-01-02".into(),
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
        let merged = merger.merge(&[timeline.clone()]).unwrap();
        assert_eq!(merged.years.len(), 1);
        let year = &merged.years[0];
        assert_eq!(year.year, "2020");
        assert_eq!(year.total, 1234);
        assert_eq!(year.range.start, "2020-01-01");
        assert_eq!(year.range.end, "2020-01-02");
    }

    #[test]
    fn test_merge_multiple() {
        let mut timeline1 = Timeline::default();
        let mut timeline2 = Timeline::default();

        let range1 = Range {
            start: "2020-01-01".into(),
            end: "2020-01-02".into(),
        };
        let range2 = Range {
            start: "2020-01-01".into(),
            end: "2020-01-03".into(),
        };

        let year1 = Year {
            year: "2020".into(),
            total: 1234,
            range: range1,
        };
        timeline1.years = vec![year1];

        let year2 = Year {
            year: "2020".into(),
            total: 6,
            range: range2,
        };
        timeline2.years = vec![year2];

        let contributions1 = vec![
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

        let contributions2 = vec![
            Contribution {
                date: "2020-01-01".into(),
                count: 0,
                color: "".into(),
                intensity: 0,
            },
            Contribution {
                date: "2020-01-02".into(),
                count: 5,
                color: "".into(),
                intensity: 4,
            },
            Contribution {
                date: "2020-01-03".into(),
                count: 1,
                color: "".into(),
                intensity: 1,
            },
        ];

        timeline1.contributions = contributions1.clone();
        timeline2.contributions = contributions2.clone();

        let mut merger = Merger::new();
        let merged = merger
            .merge(&[timeline1.clone(), timeline2.clone()])
            .unwrap();
        assert_eq!(merged.years.len(), 1);
        let year = &merged.years[0];
        assert_eq!(year.year, "2020");
        assert_eq!(year.total, 1240);
        assert_eq!(year.range.start, "2020-01-01");
        assert_eq!(year.range.end, "2020-01-03");
    }
}
