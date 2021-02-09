use anyhow::{Context, Result};
use quantiles::ckms::CKMS;

/// Each cell in the timeline is shaded with one of 5 possible colors. These
/// colors correspond to the quartiles of the normal distribution over the range
/// [0, max(v)] where v is the sum of issues opened, pull requests proposed and
/// commits authored per day.
/// https://bd808.com/blog/2013/04/17/hacking-github-contributions-calendar/
/// https://github.community/t/the-color-coding-of-contribution-graph-is-showing-wrong-information/18572
pub fn quartiles(input: &[usize]) -> Result<Vec<usize>> {
    let max = input
        .iter()
        .max()
        .with_context(|| format!("Cannot get maximum from input {:?}", input))?;

    let mut ckms = CKMS::<u32>::new(0.001);

    // Note that we don't insert the actual input values themselves,
    // but the range from 0 to max(input)
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

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn test_quartiles() {
        let input = [0, 1, 2, 3, 4, 5, 100];
        let actual = quartiles(&input).unwrap();
        let expected = vec![0, 1, 25, 50, 75];
        assert_eq!(actual, expected);
    }
}
