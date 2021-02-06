use crate::types::{Contribution, Contributions, Day, Range, Timeline, Year};
use anyhow::{bail, Context, Result};
use chrono::prelude::*;
use quantiles::ckms::CKMS;
use std::collections::{HashMap, HashSet};

/// A parser that converts git log output
/// into the JSON format understood by the
/// API of codeprints.dev.
pub struct Parser {
    input: String,
    days: HashSet<Day>,
    years_map: HashMap<i32, Year>,
    quartiles_map: HashMap<i32, CKMS<u32>>,
}

impl Parser {
    pub fn new(input: String) -> Self {
        let years_map = HashMap::new();
        let quartiles_map = HashMap::new();
        let days = HashSet::new();

        Parser {
            input,
            days,
            years_map,
            quartiles_map,
        }
    }

    fn get_quartiles(&self, year: i32) -> Result<Vec<usize>> {
        let quantiles = self
            .quartiles_map
            .get(&year)
            .context(format!("Cannot get quartiles map for {}", year))?;

        Ok(vec![
            0,
            1,
            quantiles.query(0.25).context("Cannot get quartile")?.0,
            quantiles.query(0.5).context("Cannot get quartile")?.0,
            quantiles.query(0.75).context("Cannot get quartile")?.0,
        ])
    }

    // Each cell in the timeline is shaded with one of 5 possible colors. These
    // colors correspond to the quartiles of the normal distribution over the range
    // [0, max(v)] where v is the sum of issues opened, pull requests proposed and
    // commits authored per day.
    // https://bd808.com/blog/2013/04/17/hacking-github-contributions-calendar/
    // https://github.community/t/the-color-coding-of-contribution-graph-is-showing-wrong-information/18572
    fn get_intensity(&self, day: &Day) -> Result<usize> {
        let year = day.date.year();
        let quartiles = self.get_quartiles(year)?;
        for (index, quartile) in quartiles.iter().enumerate() {
            if day.commits < *quartile {
                return Ok(index - 1);
            }
        }
        return Ok(quartiles.len() - 1);
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

    fn parse_contributions(&self) -> Result<Contributions> {
        let mut contributions = Vec::new();
        for day in &self.days {
            let intensity = self.get_intensity(&day)?;
            let color = Parser::map_color(intensity);

            contributions.push(Contribution {
                date: day.date.format("%Y-%m-%d").to_string(),
                count: day.commits,
                color,
                intensity,
            });
        }
        Ok(contributions)
    }

    fn parse_day(&self, line: &str) -> Result<Option<Day>> {
        if line.trim().is_empty() {
            // Empty lines are allowed, but skipped
            return Ok(None);
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 2 {
            bail!(
                "Invalid input line `{}`. Expected 2 input fields, got {}",
                line,
                fields.len()
            );
        };
        let commits = fields[0].parse::<usize>()?;
        let date: NaiveDate = fields[1].parse()?;
        Ok(Some(Day::new(commits, date)))
    }

    fn update_years(&mut self, day: Day) {
        let y = day.date.year();
        let mut year = self.years_map.entry(y).or_insert(Year {
            year: y.to_string(),
            total: 0,
            range: Range::default(),
        });
        year.total += day.commits;

        let date = day.date.format("%Y-%m-%d").to_string();
        if year.range.start.is_empty() || date < year.range.start {
            year.range.start = date.clone();
        }
        if year.range.end.is_empty() || date > year.range.end {
            year.range.end = date
        }
    }

    fn update_quartiles(&mut self, day: &Day) {
        let y = day.date.year();
        let year = self
            .quartiles_map
            .entry(y)
            .or_insert(CKMS::<u32>::new(0.001));
        year.insert(day.commits as u32);
    }

    pub fn update_stats(&mut self, day: Day) {
        self.update_quartiles(&day);
        self.update_years(day);
    }

    pub fn parse(&mut self) -> Result<Timeline> {
        let input = self.input.clone();
        for line in input.lines() {
            let day = self.parse_day(&line)?;
            if let Some(d) = day {
                self.days.insert(d.clone());
                self.update_stats(d);
            }
        }

        let mut years: Vec<Year> = self.years_map.iter().map(|(_k, v)| v).cloned().collect();
        years.sort();
        years.reverse();

        let contributions = self.parse_contributions()?;

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
            2 2020-04-15
            1 2020-04-16
            4 2020-04-17
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
    fn test_quartiles() {
        let input = r###"
            0 2020-04-15
            1 2020-04-16
            2 2020-04-17
            3 2020-04-18
            4 2020-04-19
            5 2020-04-20
            100 2020-04-21
        "###;
        let mut parser = Parser::new(input.to_string());
        parser.parse().unwrap();

        let actual = parser.get_quartiles(2020).unwrap();
        let expected = vec![0, 1, 25, 50, 75];
        assert_eq!(actual, expected);
    }

    // #[test]
    // fn test_quartiles_torvalds() {
    //     let raw = fs::read_to_string("fixtures/torvalds-2019-git.txt").unwrap();
    //     let lines = Parser::parse_lines(&raw).unwrap();
    //     let input = lines.iter().map(|line| line.commits).collect::<Vec<_>>();
    //     let actual = Parser::parse_quartiles(&input).unwrap();
    //     let expected = [0, 1, 11, 22, 32];
    //     assert_eq!(actual, expected);
    // }

    // #[test]
    // fn test_intensities() {
    //     let quartiles = [0, 1, 11, 22, 32];
    //     assert_eq!(0, Parser::get_intensity(&quartiles, 0));
    //     assert_eq!(1, Parser::get_intensity(&quartiles, 1));
    //     assert_eq!(1, Parser::get_intensity(&quartiles, 10));
    //     assert_eq!(2, Parser::get_intensity(&quartiles, 18));
    //     assert_eq!(3, Parser::get_intensity(&quartiles, 22));
    //     assert_eq!(4, Parser::get_intensity(&quartiles, 32));
    //     assert_eq!(4, Parser::get_intensity(&quartiles, 100));
    // }
}
