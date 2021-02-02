mod parser;
mod types;

use anyhow::{Context, Result};
use duct::cmd;
use parser::Parser;
use std::fs;

fn get_commits() -> Result<String> {
    Ok(cmd!("git", "log", "--date=short", "--pretty=format:%ad")
        .pipe(cmd!("sort"))
        .pipe(cmd!("uniq", "-c"))
        .read()?)
}

fn main() -> Result<()> {
    let input = get_commits().context("Cannot read project history")?;
    let mut parser = Parser::new(input);
    let timeline = parser.parse()?;
    let output = serde_json::to_string_pretty(&timeline)?;
    fs::write("codeprints.json", output)?;

    Ok(())
}
