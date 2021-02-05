use crate::types::{Contribution, Contributions, Day, Range, Timeline, Year};
use anyhow::{bail, Context, Result};
use chrono::prelude::*;
use quantiles::ckms::CKMS;
use std::collections::{HashMap, HashSet};

/// A parser that converts git log output into the JSON format understood by the
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

    // {
    //     "year": "2020",
    //     "total": 2661,
    //     "range": {
    //         "start": "2020-01-01",
    //         "end": "2020-12-31"
    //     }
    // }
    // fn parse_years(map: &HashMap<i32, Days>) -> Result<Years> {
    //     let mut years = vec![];
    //     for (year, days) in map {
    //         let start = days
    //             .iter()
    //             .map(|d| d.date)
    //             .min()
    //             .with_context(|| format!("Cannot read min day for {}", year))?;
    //         let end = days
    //             .iter()
    //             .map(|d| d.date)
    //             .max()
    //             .with_context(|| format!("Cannot read max day for {}", year))?;

    //         let range = Range {
    //             start: start.format("%Y-%m-%d").to_string(),
    //             end: end.format("%Y-%m-%d").to_string(),
    //         };

    //         let year_obj = Year {
    //             year: year.to_string(),
    //             total: days.iter().map(|d| d.commits).sum(),
    //             range: range,
    //         };
    //         years.push(year_obj);
    //     }
    //     Ok(years)
    // }

    fn get_intensity(&self, day: &Day) -> Result<usize> {
        let year = day.date.year();
        let ckms = self
            .quartiles_map
            .get(&year)
            .context(format!("Cannot get quartiles map for {}", year))?;

        let quartiles = [
            0,
            1,
            ckms.query(0.25).context("Cannot get quartile")?.0,
            ckms.query(0.5).context("Cannot get quartile")?.0,
            ckms.query(0.75).context("Cannot get quartile")?.0,
        ];

        for (index, quartile) in quartiles.iter().enumerate() {
            if day.commits < *quartile {
                return Ok(index - 1);
            }
        }
        return Ok(quartiles.len() - 1);
    }

    fn map_color(intensity: usize) -> String {
        let color = match intensity {
            1 => "var(--color-calendar-graph-day-L1-bg)",
            2 => "var(--color-calendar-graph-day-L2-bg)",
            3 => "var(--color-calendar-graph-day-L3-bg)",
            4 => "var(--color-calendar-graph-day-L4-bg)",
            _ => "var(--color-calendar-graph-day-bg)",
        };
        color.to_string()
    }

    fn parse_contributions(&self) -> Result<Contributions> {
        let mut contributions = Vec::new();
        for day in &self.days {
            let intensity = self.get_intensity(&day)?;
            let color = Parser::map_color(intensity);

            let contribution = Contribution {
                date: day.date.format("%Y-%m-%d").to_string(),
                count: day.commits,
                color,
                intensity,
            };
            contributions.push(contribution);
        }
        Ok(contributions)
    }

    // Each cell in the timeline is shaded with one of 5 possible colors. These
    // colors correspond to the quartiles of the normal distribution over the range
    // [0, max(v)] where v is the sum of issues opened, pull requests proposed and
    // commits authored per day.
    // https://bd808.com/blog/2013/04/17/hacking-github-contributions-calendar/
    // https://github.community/t/the-color-coding-of-contribution-graph-is-showing-wrong-information/18572
    // fn quartiles(&self, input: &[Day]) -> Result<Vec<usize>> {
    //     let max = input
    //         .iter()
    //         .map(|d| d.commits)
    //         .max()
    //         .with_context(|| format!("Cannot get maximum from input {:?}", input))?;

    //     let mut ckms = CKMS::<u32>::new(0.001);
    //     for i in 0..max {
    //         ckms.insert(i as u32);
    //     }
    //     Ok(vec![
    //         0,
    //         1,
    //         ckms.query(0.25).context("Cannot get quartile")?.0,
    //         ckms.query(0.5).context("Cannot get quartile")?.0,
    //         ckms.query(0.75).context("Cannot get quartile")?.0,
    //     ])
    // }

    fn parse_fields(&self, line: &str) -> Result<(usize, NaiveDate)> {
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

        Ok((commits, date))
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

    /// Parses the following input `2 2020-04-15`
    /// where `2` is the number of commits and `2020-04-1` is the date of the commits
    fn parse_day(&mut self, line: &str) -> Result<Day> {
        let (commits, date) = self.parse_fields(line)?;
        let day: Day = Day::new(commits, date);
        Ok(day)
    }

    pub fn update_stats(&mut self, day: Day) -> Result<()> {
        self.update_quartiles(&day);
        self.update_years(day);
        Ok(())
    }

    pub fn parse(&mut self) -> Result<Timeline> {
        let input = self.input.clone();
        for line in input.lines() {
            let day = self.parse_day(&line)?;
            self.days.insert(day.clone());
            self.update_stats(day)
                .context("Cannot update stats for {:?}")?;
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
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_years() {
        let days: Vec<Day> = vec![
            Day::new(2, NaiveDate::from_ymd(2020, 4, 15)),
            Day::new(1, NaiveDate::from_ymd(2020, 4, 16)),
            Day::new(4, NaiveDate::from_ymd(2020, 4, 17)),
        ];
        let map = Parser::map_years(days.clone());
        let output = Parser::parse_years(map).unwrap();

        let range = Range {
            start: "2020-04-15".to_string(),
            end: "2020-04-17".to_string(),
        };
        let expected = vec![Year {
            year: "2020".to_string(),
            total: 7,
            range: range,
        }];
        assert_eq!(output, expected);
    }

    #[test]
    fn test_quartiles() {
        let input = [0, 1, 2, 3, 4, 5, 100];
        let actual = Parser::parse_quartiles(&input).unwrap();
        let expected = vec![0, 1, 25, 50, 75];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_quartiles_torvalds() {
        let raw = fs::read_to_string("fixtures/torvalds-2019-git.txt").unwrap();
        let lines = Parser::parse_lines(&raw).unwrap();
        let input = lines.iter().map(|line| line.commits).collect::<Vec<_>>();
        let actual = Parser::parse_quartiles(&input).unwrap();
        let expected = [0, 1, 11, 22, 32];
        assert_eq!(actual, expected);
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
