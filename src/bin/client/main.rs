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
use client::count_commits;
use client::Parser;
use std::fs;
use structopt::StructOpt;

mod options;
use options::Opt;

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let input = count_commits(opt.before, opt.after, opt.author, opt.committer)
        .context("Cannot read project history")?;
    let mut parser = Parser::new(input);
    let timeline = parser.parse()?;
    let output = serde_json::to_string_pretty(&timeline)?;
    fs::write("codeprints.json", output)?;

    Ok(())
}
