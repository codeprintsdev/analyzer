mod json;

use anyhow::{bail, Context, Result};
use chrono::prelude::*;
use duct::cmd;
use json::{Contribution, Contributions, Range, Timeline, Year, Years};
use quantiles::ckms::CKMS;
use std::{collections::HashMap, fs};

#[derive(Debug, PartialEq, Clone)]
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

fn map_years(days: Days) -> HashMap<i32, Days> {
    let mut years_map = HashMap::new();
    for day in days {
        let y = day.date.year();
        let year = years_map.entry(y).or_insert(vec![]);
        year.push(day);
    }
    years_map
}

// {
//     "year": "2020",
//     "total": 2661,
//     "range": {
//         "start": "2020-01-01",
//         "end": "2020-12-31"
//     }
// }
fn parse_years(map: HashMap<i32, Days>) -> Result<Years> {
    let mut years = vec![];
    for (year, days) in map {
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

fn get_intensity(quartiles: &[usize], count: usize) -> usize {
    for (index, quartile) in quartiles.iter().enumerate() {
        if count < *quartile {
            return index - 1;
        }
    }
    return quartiles.len() - 1;
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

fn parse_contributions(
    quartiles_map: HashMap<i32, Vec<usize>>,
    days: Days,
) -> Result<Contributions> {
    let mut contributions = Vec::new();
    for day in days {
        let y = day.date.year();
        let count = day.commits;
        let intensity = get_intensity(&quartiles_map[&y], count);
        let color = map_color(intensity);

        let contribution = Contribution {
            date: day.date.format("%Y-%m-%d").to_string(),
            count,
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
fn parse_quartiles(input: &[usize]) -> Result<Vec<usize>> {
    let max = input
        .iter()
        .max()
        .with_context(|| format!("Cannot get maximum from input {:?}", input))?;

    let mut ckms = CKMS::<u32>::new(0.001);
    for i in 0..*max {
        ckms.insert(i as u32);
    }
    Ok(vec![
        0,
        1,
        ckms.query(0.25).context("Cannot get quartile")?.0,
        ckms.query(0.5).context("Cannot get quartile")?.0,
        ckms.query(0.75).context("Cannot get quartile")?.0,
    ])
}

fn parse_lines(raw: String) -> Result<Days> {
    raw.lines().map(|line| parse_day(line)).collect()
}

fn main() -> Result<()> {
    let output = get_commits().context("Cannot read project history")?;
    let days: Result<Days> = parse_lines(output);
    let days = days?;
    let map = map_years(days.clone());

    let mut quartiles_map = HashMap::new();
    for (year, days) in map.clone() {
        let input: Vec<usize> = days.iter().map(|d| d.commits).collect();
        let quartiles = parse_quartiles(&input)?;
        quartiles_map.insert(year, quartiles);
    }
    let contributions = parse_contributions(quartiles_map, days)?;
    let years = parse_years(map)?;

    let timeline = Timeline {
        years,
        contributions,
    };

    let output = serde_json::to_string_pretty(&timeline)?;
    fs::write("codeprints.json", output)?;

    Ok(())
}

#[cfg(test)]
mod test_super {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_line() {
        let expected = Day::new(123, NaiveDate::from_ymd(2020, 11, 04));
        let actual = parse_day("123 2020-11-04").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_years() {
        let days: Vec<Day> = vec![
            Day::new(2, NaiveDate::from_ymd(2020, 4, 15)),
            Day::new(1, NaiveDate::from_ymd(2020, 4, 16)),
            Day::new(4, NaiveDate::from_ymd(2020, 4, 17)),
        ];
        let map = map_years(days.clone());
        let output = parse_years(map).unwrap();

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
        let actual = parse_quartiles(&input).unwrap();
        let expected = vec![0, 1, 25, 50, 75];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_quartiles_torvalds() {
        let raw = fs::read_to_string("fixtures/torvalds-2019-git.txt").unwrap();
        let lines = parse_lines(raw).unwrap();
        let input = lines.iter().map(|line| line.commits).collect::<Vec<_>>();
        let actual = parse_quartiles(&input).unwrap();
        let expected = [0, 1, 11, 22, 32];
        assert_eq!(actual, expected);
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
