use crate::types::{Contribution, Contributions, Range, Timeline, Year};
use crate::{git, quartiles::quartiles};
use anyhow::Result;
use chrono::prelude::*;
use std::{cmp::{max, min}, collections::HashMap, convert::TryFrom};

/// The internal state of the parser
#[derive(Debug, Default)]
pub struct ParseState {
    years_map: HashMap<i32, Year>,
    days: HashMap<NaiveDate, usize>,
    before: Option<NaiveDate>,
    after: Option<NaiveDate>,
}

impl ParseState {
    /// Add a single day to the map of years
    pub fn update_years(&mut self, date: NaiveDate) {
        let y = date.year();
        let start = match self.before {
            Some(before) => max(before, date),
            None => date,
        };
        let end = match self.after {
            Some(after) => min(after, date),
            None => date,
        };
        let mut year = self.years_map.entry(y).or_insert(Year {
            year: y.to_string(),
            total: 0,
            // range: Range::default,
            // Always show full year
            range: Range {
                start: start.format("%y-%m-%d").to_string(),
                end: end.format("%y-%m-%d").to_string(),
            },
        });
        year.total += 1;

        // let date = date.format("%Y-%m-%d").to_string();
        // if year.range.start.is_empty() || date < year.range.start {
        //     year.range.start = date.clone();
        // }
        // if year.range.end.is_empty() || date > year.range.end {
        //     year.range.end = date
        // }
    }

    /// Add a single day to the map of days
    pub fn update_days(&mut self, date: NaiveDate) {
        *self.days.entry(date).or_insert(0) += 1;
    }
}

/// Backfill missing days with zero commits
fn backfill(year: i32, days: &mut HashMap<NaiveDate, usize>) {
    for d in NaiveDate::from_ymd(year, 1, 1).iter_days() {
        if d.year() != year {
            break;
        }
        days.entry(d).or_insert(0);
    }
}

// Each cell in the timeline is shaded with one of 5 possible colors. These
// colors correspond to the quartiles of the normal distribution over the range
// [0, max(v)] where v is the sum of issues opened, pull requests proposed and
// commits authored per day.
// https://bd808.com/blog/2013/04/17/hacking-github-contributions-calendar/
// https://github.community/t/the-color-coding-of-contribution-graph-is-showing-wrong-information/18572
fn get_intensity(quartiles: &[usize], commits: usize) -> usize {
    for (index, quartile) in quartiles.iter().enumerate() {
        if commits < *quartile {
            return index - 1;
        }
    }
    quartiles.len() - 1
}

fn map_color(intensity: usize) -> String {
    match intensity {
        1 => "var(--color-calendar-graph-day-L1-bg)",
        2 => "var(--color-calendar-graph-day-L2-bg)",
        3 => "var(--color-calendar-graph-day-L3-bg)",
        4 => "var(--color-calendar-graph-day-L4-bg)",
        _ => "var(--color-calendar-graph-day-bg)",
    }
    .to_string()
}

fn create_contributions(
    days: &HashMap<NaiveDate, usize>,
    quartiles: &[usize],
) -> Vec<Contribution> {
    let mut contributions = Vec::new();
    for (date, commits) in days {
        let intensity = get_intensity(&quartiles, *commits);
        let color = map_color(intensity);

        contributions.push(Contribution {
            date: date.format("%Y-%m-%d").to_string(),
            count: *commits,
            color,
            intensity,
        });
    }
    contributions
}

impl TryFrom<&ParseState> for Timeline {
    type Error = anyhow::Error;
    fn try_from(state: &ParseState) -> Result<Self> {
        let mut years: Vec<Year> = state.years_map.values().cloned().collect();
        years.sort();
        years.reverse();

        let mut contributions = Contributions::new();
        for year in &years {
            let mut year_contribs: HashMap<NaiveDate, usize> = state
                .days
                .clone()
                .into_iter()
                .filter(|(date, _commits)| date.year().to_string() == year.year)
                .collect();
            backfill(year.year.parse::<i32>()?, &mut year_contribs);
            let commits: Vec<usize> = year_contribs.values().cloned().collect();
            let quartiles = quartiles(&commits)?;

            let mut contribs = create_contributions(&year_contribs, &quartiles);
            contributions.append(&mut contribs);
        }
        contributions.sort();

        Ok(Timeline {
            years,
            contributions,
        })
    }
}

/// A parser that converts git log output
/// into the JSON format understood by the
/// API of codeprints.dev.
#[derive(Debug, Default)]
pub struct Parser {
    input: String,
    state: ParseState,
}

impl Parser {
    /// Create a new parser that analyzes the given input
    pub fn new(input: String) -> Self {
        Parser {
            input,
            ..Default::default()
        }
    }

    /// Set the minimum date of the timeline. Commits older than the given date
    /// will not be counted and won't be in the final output
    pub fn set_before(&mut self, before: String) -> Result<&mut Self> {
        let before = git::parse_date(&before)?;
        if let Some(before) = before {
            self.state.before = Some(before);
        }
        Ok(self)
    }

    /// Set the maximum date of the timeline. Commits newer than the given date
    /// will not be counted and won't be in the final output
    pub fn set_after(&mut self, after: String) -> Result<&mut Self> {
        let after = git::parse_date(&after)?;
        if let Some(after) = after {
            self.state.after = Some(after);
        }
        Ok(self)
    }

    fn outside_date_range(&self, day: NaiveDate) -> bool {
        if let Some(before) = self.state.before {
            if day < before {
                return true;
            }
        }
        if let Some(after) = self.state.after {
            if day > after {
                return true;
            }
        }
        false
    }

    /// Execute the parsing step
    pub fn parse(&mut self) -> Result<Timeline> {
        let input = self.input.clone();
        for line in input.lines() {
            let day = git::parse_date(&line)?;
            if let Some(d) = day {
                if self.outside_date_range(d) {
                    continue;
                }
                self.state.update_days(d);
                self.state.update_years(d);
            }
        }

        Ok(Timeline::try_from(&self.state)?)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_parse_years() {
        let input = r###"
            2020-04-15
            2020-04-15
            2020-04-16
            2020-04-17
            2020-04-17
            2020-04-17
            2020-04-17
        "###;

        let range = Range {
            start: "2020-04-15".to_string(),
            end: "2020-04-17".to_string(),
        };
        let expected = vec![Year {
            year: "2020".to_string(),
            total: 7,
            range: range,
        }];
        let mut parser = Parser::new(input.to_string());
        assert_eq!(parser.parse().unwrap().years, expected);
    }

    #[test]
    fn test_intensities() {
        let quartiles = [0, 1, 11, 22, 32];
        assert_eq!(0, get_intensity(&quartiles, 0));
        assert_eq!(1, get_intensity(&quartiles, 1));
        assert_eq!(1, get_intensity(&quartiles, 10));
        assert_eq!(2, get_intensity(&quartiles, 18));
        assert_eq!(3, get_intensity(&quartiles, 22));
        assert_eq!(4, get_intensity(&quartiles, 32));
        assert_eq!(4, get_intensity(&quartiles, 100));
    }
}
