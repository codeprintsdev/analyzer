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
use options::Command;

const OUTPUT_FILE: &'static str = "codeprints.json";

fn main() -> Result<()> {
    let opt = Command::from_args();

    match opt {
        Command::Run {
            before,
            after,
            author,
            committer,
        } => {
            print!("Analyzing commits in current repository...");
            let input = count_commits(&before, &after, author, committer).context(
                "Cannot read project history. Make sure there is no typo in the command",
            )?;
            let mut parser = Parser::new(input);
            if let Some(before) = before {
                parser.set_before(before)?;
            }
            if let Some(after) = after {
                parser.set_after(after)?;
            }
            let timeline = parser.parse()?;
            let output = serde_json::to_string_pretty(&timeline)?;
            fs::write(OUTPUT_FILE, output)?;
            println!("done!");
            println!("Output file: {}", OUTPUT_FILE);
        }
        Command::Merge {} => {
            // Find all `codeprints*.json` files in the current directory
            // using glob.
            // Read each one into memory
            // Merge the results together
            // Write a `codeprints_merged.json` file
            unimplemented!();
        }
    };
    Ok(())
}
