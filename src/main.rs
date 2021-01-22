use anyhow::{bail, Context, Result};
use duct::cmd;

fn get_commits() -> Result<String> {
    Ok(cmd!("git", "log", "--date=short", "--pretty=format:%ad")
        .pipe(cmd!("sort"))
        .pipe(cmd!("uniq", "-c"))
        .read()?)
}

fn main() -> Result<()> {
    let output = get_commits().context("Cannot read project history")?;
    println!("{}", output);
    Ok(())
}
