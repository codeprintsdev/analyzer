use anyhow::{Context, Result};
use chrono::NaiveDate;
use duct::cmd;

/// Get the count of commits for each day from the git logs
pub fn count_commits(
    before: &Option<String>,
    after: &Option<String>,
    authors: Vec<String>,
    committers: Vec<String>,
) -> Result<String> {
    let mut args = vec!["log", "--date=short", "--pretty=format:%ad"];
    if let Some(before) = &before {
        args.push("--before");
        args.push(before);
    }
    if let Some(after) = &after {
        args.push("--after");
        args.push(after);
    }
    for author in &authors {
        args.push("--author");
        args.push(author);
    }
    for committer in &committers {
        args.push("--committer");
        args.push(committer);
    }
    let commits = cmd("git", &args).read()?;
    Ok(commits)
}

/// Parse a date from the git log
pub fn parse_date(line: &str) -> Result<Option<NaiveDate>> {
    if line.trim().is_empty() {
        // Empty lines are allowed, but skipped
        return Ok(None);
    }
    let date: NaiveDate = line.parse().context(format!("Invalid date {}", line))?;
    Ok(Some(date))
}

/// Get the current git commit sha
pub fn sha() -> Result<String> {
    let sha = cmd("git", &["rev-parse", "--short", "HEAD"]).read()?;
    Ok(sha)
}
