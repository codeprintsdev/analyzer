use crate::quartiles::quartiles;
use crate::types::{Contribution, Contributions, Range, Timeline, Year};
use anyhow::{Context, Result};
use chrono::prelude::*;
use std::collections::HashMap;

/// A parser that converts git log output
/// into the JSON format understood by the
/// API of codeprints.dev.
#[derive(Debug)]
pub struct Parser {
    input: String,
    years_map: HashMap<i32, Year>,
    days: HashMap<NaiveDate, usize>,
}

impl Parser {
    /// Create a new parser that analyzes the given input
    pub fn new(input: String) -> Self {
        let years_map = HashMap::new();
        let days = HashMap::new();

        Parser {
            input,
            years_map,
            days,
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

    /// Backfill missing days with zero commits
    fn backfill(year: i32, days: &mut HashMap<NaiveDate, usize>) {
        for d in NaiveDate::from_ymd(year, 1, 1).iter_days() {
            if d == NaiveDate::from_ymd(year + 1, 1, 1) {
                break;
            }
            days.entry(d).or_insert(0);
        }
    }

    fn create_contributions(
        &self,
        days: &HashMap<NaiveDate, usize>,
        quartiles: &[usize],
    ) -> Vec<Contribution> {
        let mut contributions = Vec::new();
        for (date, commits) in days {
            let intensity = Self::get_intensity(&quartiles, *commits);
            let color = Self::map_color(intensity);

            contributions.push(Contribution {
                date: date.format("%Y-%m-%d").to_string(),
                count: *commits,
                color,
                intensity,
            });
        }
        contributions
    }

    fn parse_date(&self, line: &str) -> Result<Option<NaiveDate>> {
        if line.trim().is_empty() {
            // Empty lines are allowed, but skipped
            return Ok(None);
        }
        let date: NaiveDate = line.parse().context(format!("Invalid date {}", line))?;
        Ok(Some(date))
    }

    /// Add a single day to the map of years
    fn update_years(&mut self, date: NaiveDate) {
        let y = date.year();
        let mut year = self.years_map.entry(y).or_insert(Year {
            year: y.to_string(),
            total: 0,
            range: Range::default(),
        });
        year.total += 1;

        let date = date.format("%Y-%m-%d").to_string();
        if year.range.start.is_empty() || date < year.range.start {
            year.range.start = date.clone();
        }
        if year.range.end.is_empty() || date > year.range.end {
            year.range.end = date
        }
    }

    /// Add a single day to the map of days
    fn update_days(&mut self, date: NaiveDate) {
        *self.days.entry(date).or_insert(0) += 1;
    }

    /// Execute the parsing step
    pub fn parse(&mut self) -> Result<Timeline> {
        let input = self.input.clone();
        for line in input.lines() {
            let day = self.parse_date(&line)?;
            if let Some(d) = day {
                self.update_days(d);
                self.update_years(d);
            }
        }

        let mut years: Vec<Year> = self.years_map.values().cloned().collect();
        years.sort();
        years.reverse();

        let mut contributions = Contributions::new();
        for year in &years {
            let mut year_contribs: HashMap<NaiveDate, usize> = self
                .days
                .clone()
                .into_iter()
                .filter(|(date, _commits)| date.year().to_string() == year.year)
                .collect();
            Self::backfill(year.year.parse::<i32>()?, &mut year_contribs);
            let commits: Vec<usize> = year_contribs.values().cloned().collect();
            let quartiles = quartiles(&commits)?;

            let mut contribs = self.create_contributions(&year_contribs, &quartiles);
            contributions.append(&mut contribs);
        }
        contributions.sort();

        Ok(Timeline {
            years,
            contributions,
        })
    }
}

#[cfg(test)]
mod test_super {

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
        assert_eq!(0, Parser::get_intensity(&quartiles, 0));
        assert_eq!(1, Parser::get_intensity(&quartiles, 1));
        assert_eq!(1, Parser::get_intensity(&quartiles, 10));
        assert_eq!(2, Parser::get_intensity(&quartiles, 18));
        assert_eq!(3, Parser::get_intensity(&quartiles, 22));
        assert_eq!(4, Parser::get_intensity(&quartiles, 32));
        assert_eq!(4, Parser::get_intensity(&quartiles, 100));
    }
}
