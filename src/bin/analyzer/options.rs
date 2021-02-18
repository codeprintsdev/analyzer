use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "codeprints-analyzer")]
pub enum Command {
    Run {
        /// Limit the commits output to ones with author header lines
        /// that match the specified pattern.
        /// This is passed verbatim to git. See `git log --help` for more info.
        #[structopt(short, long)]
        author: Vec<String>,

        /// Limit the commits output to ones with committer header lines
        /// that match the specified pattern.
        /// This is passed verbatim to git. See `git log --help` for more info.
        #[structopt(short, long)]
        committer: Vec<String>,

        // Show commits older than a specific date.
        #[structopt(alias = "until", long)]
        before: Option<String>,

        // Show commits more recent than a specific date.
        #[structopt(alias = "since", long)]
        after: Option<String>,
    },
    Merge {},
}
