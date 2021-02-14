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
use codeprints_analyzer::count_commits;
use codeprints_analyzer::Parser;
use std::fs;
use structopt::StructOpt;

mod options;
use options::Opt;

const OUTPUT_FILE: &'static str = "codeprints.json";

fn main() -> Result<()> {
    let opt = Opt::from_args();

    print!("Analyzing commits in current repository...");
    let input = count_commits(opt.before, opt.after, opt.author, opt.committer)
        .context("Cannot read project history. Make sure there is no typo in the command")?;
    let mut parser = Parser::new(input);
    let timeline = parser.parse()?;
    let output = serde_json::to_string_pretty(&timeline)?;
    fs::write(OUTPUT_FILE, output)?;
    println!("done!");
    println!("Output file: {}", OUTPUT_FILE);
    Ok(())
}
