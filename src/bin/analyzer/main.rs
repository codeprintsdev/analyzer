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
use codeprints_analyzer::git;
use codeprints_analyzer::Merger;
use codeprints_analyzer::Parser;
use codeprints_analyzer::Timeline;
use glob::glob;
use std::fs;
use structopt::StructOpt;

mod options;
use options::Command;

fn write(timeline: &Timeline, output_file: &str) -> Result<()> {
    let output = serde_json::to_string_pretty(&timeline)?;
    fs::write(output_file, output)?;
    println!("done!");
    println!("Output file: {}", output_file);
    Ok(())
}

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
            let input = git::count_commits(&before, &after, author, committer).context(
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

            let sha = git::sha()?;
            write(&timeline, &format!("codeprints_{}.json", sha))?;
        }
        Command::Merge {} => {
            // Find all `codeprints*.json` files in the current directory
            // using glob.
            let mut merger = Merger::new();
            for entry in glob("codeprints*.json")? {
                match entry {
                    Ok(path) => {
                        println!("Merging {}", path.display());
                        let input = fs::read_to_string(path)?;
                        let mut parser = Parser::new(input);
                        let timeline = parser.parse()?;
                        merger.merge_timeline(&timeline)?;
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
            write(&merger.timeline()?, "codeprints_merged.json")?;
        }
    };
    Ok(())
}
