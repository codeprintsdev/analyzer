mod json;

use anyhow::{bail, Context, Result};
use chrono::prelude::*;
use duct::cmd;
use json::{Range, Year, Years};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
struct Day {
    commits: usize,
    date: NaiveDate,
}

impl Day {
    fn new(commits: usize, date: NaiveDate) -> Self {
        Day { commits, date }
    }
}

type Days = Vec<Day>;

fn get_commits() -> Result<String> {
    Ok(cmd!("git", "log", "--date=short", "--pretty=format:%ad")
        .pipe(cmd!("sort"))
        .pipe(cmd!("uniq", "-c"))
        .read()?)
}

/// Parses the following input `2 2020-04-15`
/// where `2` is the number of commits and `2020-04-1` is the date of the commits
fn parse_day(line: &str) -> Result<Day> {
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

    let parsed: Day = Day::new(commits, date);
    Ok(parsed)
}

// {
//     "year": "2020",
//     "total": 2661,
//     "range": {
//         "start": "2020-01-01",
//         "end": "2020-12-31"
//     }
// }
fn parse_years(days: Days) -> Result<Vec<Year>> {
    let mut years_map = HashMap::new();
    for day in days {
        let y = day.date.year();
        let year = years_map.entry(y).or_insert(vec![]);
        year.push(day);
    }
    let mut years = vec![];
    for (year, days) in years_map {
        let start = days
            .iter()
            .map(|d| d.date)
            .min()
            .with_context(|| format!("Cannot read min day for {}", year))?;
        let end = days
            .iter()
            .map(|d| d.date)
            .max()
            .with_context(|| format!("Cannot read max day for {}", year))?;

        let range = Range {
            start: start.format("%Y-%m-%d").to_string(),
            end: end.format("%Y-%m-%d").to_string(),
        };

        let year_obj = Year {
            year: year.to_string(),
            total: days.iter().map(|d| d.commits).sum(),
            range: range,
        };
        years.push(year_obj);
    }
    Ok(years)
}

fn parse() -> Result<Years> {
    let output = get_commits().context("Cannot read project history")?;
    let days: Result<Days> = output.lines().map(|line| parse_day(line)).collect();
    Ok(parse_years(days?)?)
}

fn main() -> Result<()> {
    let years = parse()?;
    println!("years: {:?}", years);
    Ok(())
}

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn test_parse_line() {
        let expected = Day::new(123, NaiveDate::from_ymd(2020, 11, 04));
        let actual = parse_day("123 2020-11-04").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_years() {
        let input: Vec<Day> = vec![
            Day::new(2, NaiveDate::from_ymd(2020, 4, 15)),
            Day::new(1, NaiveDate::from_ymd(2020, 4, 16)),
            Day::new(4, NaiveDate::from_ymd(2020, 4, 17)),
        ];
        let output = parse_years(input).unwrap();

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
}
