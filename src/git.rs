use anyhow::Result;
use duct::cmd;

/// Get the count of commits for each day from the git logs
pub fn count_commits(
    before: Option<String>,
    after: Option<String>,
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
