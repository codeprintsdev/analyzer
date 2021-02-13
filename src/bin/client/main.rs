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
use structopt::StructOpt;

mod options;
use options::Opt;

fn get_commits(
    before: Option<String>,
    after: Option<String>,
    authors: Vec<String>,
    commiters: Vec<String>,
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
    for committer in &commiters {
        args.push("--committer");
        args.push(committer);
    }
    let cmd = cmd("git", &args)
        .pipe(cmd!("sort"))
        .pipe(cmd!("uniq", "-c"))
        .read()?;
    Ok(cmd)
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let input = get_commits(opt.before, opt.after, opt.author, opt.committer)
        .context("Cannot read project history")?;
    let mut parser = Parser::new(input);
    let timeline = parser.parse()?;
    let output = serde_json::to_string_pretty(&timeline)?;
    fs::write("codeprints.json", output)?;

    Ok(())
}
