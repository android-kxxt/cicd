//! Changelog Generator
//!
//! This generator generates changelog for superprojects managed by `repo`.
//! To achieve that, we scrape the git log of each git repo from its original commit to
//! the target commit.
//!
//! This generator needs to run inside a repo checkout as it needs to invoke git for getting
//! all the details. (TODO: maybe use a library instead of shelling out)
//!
//! Optionally, we support excluding some repos and explicitly include some repos to create
//! for example a device-specific changelog for AOSP builds.
//!
//! And we should also report updates in manifests repo and local_manifests
//! (provided that it is a git repo)

use palc::Parser;

use crate::cli::Cli;

mod snapshot;
mod changelog;
mod template;
mod cli;
mod repo_log;

fn main() {
    let cli = Cli::parse();
    eprintln!("CLI: {cli:?}");
}
