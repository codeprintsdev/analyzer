#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use anyhow::{Context, Result};
use client::Parser;
use duct::cmd;
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
